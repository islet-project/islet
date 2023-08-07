use crate::mm::address::PhysAddr;
use crate::mm::error::Error;
use crate::mm::page_table::{self, Level};

use super::translation::{add_l1_table, addr_to_idx, map_l1_table, L0_TABLE_ENTRY_SIZE_RANGE};
use super::{GranuleState, GRANULE_SIZE};
use spinning_top::Spinlock;

extern crate alloc;
use alloc::rc::Rc;

// Safety: concurrency safety
//  - For a granule status table that manages granules, it doesn't use a big lock for efficiency. So, we need to associate "lock" with each granule entry.
//  - For granule entries and a page table for them, removing a table or an entry is not supported on purpose to make things easier for concurrency safety.
//  - There are two points that requires locking,
//    - (1) subtable creation: `set_with_page_table_flags_via_alloc()` is in charge of it. In this function, validity_check-memory_alloc-set_state should be done under the lock.
//    - (2) state change on entry: `set()` should be done under the lock.
//  - Each entry can be either table (L1 table) or granule. This is determined by `Inner.table`.

pub struct Granule {
    /// granule state
    state: u64,
    /// physical address which is aligned with GRANULE_SIZE
    addr: usize,
    /// parent that this granule points to
    /// the only case at this point is "Rd(parent) - Rec(child)"
    /// Notice: do not put self-reference into this field, which may cause undefined behaviors.
    parent: Option<Inner>,
}

impl Granule {
    fn set_state<F>(&mut self, addr: usize, state: u64, destroy_callback: F) -> Result<(), Error>
    where
        F: Fn() -> Result<(), Error>,
    {
        let prev = self.state;

        if prev == GranuleState::Undelegated {
            if state != GranuleState::Delegated {
                return Err(Error::MmStateError);
            }
        } else if prev == GranuleState::Delegated {
            if state == GranuleState::Delegated {
                return Err(Error::MmStateError);
            }
        } else {
            if state != GranuleState::Delegated {
                return Err(Error::MmStateError);
            }
        }

        // The case of destroying a granule:
        //   According to the spec, transition from P to Delegated must get contents to be wiped.
        //   TODO: when we try to zeroize() in the transition of "Undelegated->Delegated", it will face a tf-a-test failure. we need to look into this issue.
        //   Note: currently, as we don't map RTT, rule out the case where prev == RTT.
        if prev != GranuleState::Undelegated
            && prev != GranuleState::RTT
            && state == GranuleState::Delegated
        {
            // wipe out contents
            self.zeroize();

            // release parent so that parent can be destroyed
            self.parent.take();

            // transition from something to Delegatated means "destroyed". (e.g., Rd --> Delegated when REALM_DESTROY)
            destroy_callback()?;
        }

        self.state = state;
        self.addr = addr;
        Ok(())
    }

    fn set_state_with_parent(
        &mut self,
        addr: usize,
        state: u64,
        parent: Inner,
    ) -> Result<(), Error> {
        // parent-child state validation check
        // (Parent, Child): (Rd, Rec) --> only one case at this moment.
        if state != GranuleState::Rec || parent.granule.state() != GranuleState::RD {
            return Err(Error::MmStateError);
        }

        self.set_state(addr, state, || Ok(()))?;
        self.parent = Some(parent);
        Ok(())
    }

    fn set_addr(&mut self, addr: usize) {
        self.addr = addr;
    }

    fn addr(&self) -> usize {
        self.addr
    }

    fn state(&self) -> u64 {
        self.state
    }

    fn zeroize(&mut self) {
        let buf = self.addr;
        unsafe {
            core::ptr::write_bytes(buf as *mut usize, 0x0, GRANULE_SIZE / 8);
        }
    }
}

/// The rules for reference counting:
///   - all entries has 1 of refcount by default when created by new()
///   - there are two points that increment/decrement refcount:
///     (1) parent-child relationship: `get_inner()` clones it and this is put into `Granule.parent`.
///         this is to prevent a granule from getting destroyed when there is still someone pointing to it. (e.g., Rd -> Rec)
///         this refcount lasts for a long time across different RMI commands.
///     (2) for local RMI commands:
///         in some cases, refcount needs to be increased at the start of a RMI command and decreased at the end.
///         e.g., when one CPU goes in "REC_ENTER" and read REC, while another CPU goes in "REC_DESTROY" simultaneously, it may go problematic.
///               we need to increase refcount in "REC_ENTER" to prevent REC from getting destroyed in "REC_DESTROY".
pub struct Inner {
    granule: Rc<Granule>,
    table: bool,
    valid: bool,
}

impl Inner {
    fn new() -> Self {
        Self {
            granule: Rc::new(Granule {
                state: GranuleState::Undelegated,
                addr: 0,
                parent: None,
            }),
            table: false,
            valid: false,
        }
    }

    fn addr(&self) -> usize {
        self.granule.addr()
    }

    pub fn state(&self) -> u64 {
        self.granule.state()
    }

    fn set_state(&mut self, addr: PhysAddr, state: u64) -> Result<(), Error> {
        let refcount = Rc::strong_count(&self.granule);

        Rc::get_mut(&mut self.granule).map_or_else(
            || Err(Error::MmRefcountError),
            |g| {
                g.set_state(addr.as_usize(), state, || {
                    if refcount > 1 {
                        // if this is a command to destroy granule (e.g., Rd -> Delegated),
                        // there has to be no one who points to it.
                        // for example, when REALM_DESTROY, it must be guranteed that no RECs point to it.
                        Err(Error::MmIsInUse)
                    } else {
                        Ok(())
                    }
                })
            },
        )?;
        self.table = false;
        self.valid = true;
        Ok(())
    }

    fn set_state_with_inner(
        &mut self,
        addr: PhysAddr,
        state: u64,
        inner: Inner,
    ) -> Result<(), Error> {
        Rc::get_mut(&mut self.granule).map_or_else(
            || Err(Error::MmRefcountError),
            |g| g.set_state_with_parent(addr.as_usize(), state, inner),
        )
    }

    fn set_state_for_table(&mut self, index: usize) -> Result<(), Error> {
        match add_l1_table(index) {
            Ok(addr) => {
                Rc::get_mut(&mut self.granule).map_or_else(
                    || Err(Error::MmRefcountError),
                    |g| {
                        g.set_addr(addr);
                        Ok(())
                    },
                )?;
            }
            Err(e) => {
                return Err(e);
            }
        }

        self.table = true;
        self.valid = true;
        Ok(())
    }

    fn valid(&self) -> bool {
        self.valid
    }
}

impl Clone for Inner {
    fn clone(&self) -> Inner {
        Inner {
            granule: self.granule.clone(),
            table: self.table,
            valid: self.valid,
        }
    }
}

pub struct Entry(Spinlock<Inner>);
impl page_table::Entry for Entry {
    type Inner = Inner;

    fn new() -> Self {
        Self(Spinlock::new(Inner::new()))
    }

    fn is_valid(&self) -> bool {
        self.0.lock().valid()
    }

    fn clear(&mut self) {}

    fn address(&self, _level: usize) -> Option<PhysAddr> {
        Some(PhysAddr::from(self.0.lock().addr()))
    }

    fn set(&mut self, addr: PhysAddr, flags: u64) -> Result<(), Error> {
        self.0.lock().set_state(addr, flags)
    }

    fn set_with_page_table_flags(&mut self, _addr: PhysAddr) -> Result<(), Error> {
        // Note: this function is not used. To enable entry-level locking,
        // it needs to be forced to use `set_with_page_table_flags_via_alloc()`.
        Err(Error::MmErrorOthers)
    }

    fn set_with_inner(
        &mut self,
        addr: PhysAddr,
        flags: u64,
        inner: Self::Inner,
    ) -> Result<(), Error> {
        self.0.lock().set_state_with_inner(addr, flags, inner)
    }

    fn set_with_page_table_flags_via_alloc<T: FnMut() -> usize>(
        &mut self,
        index: usize,
        _alloc: T,
    ) -> Result<(), Error>
    where
        T: FnMut() -> usize,
    {
        let mut inner = self.0.lock();
        if !inner.valid() {
            inner.set_state_for_table(index)
        } else {
            Ok(())
        }
    }

    fn index<L: Level>(addr: usize) -> usize {
        match addr_to_idx(addr) {
            Ok(idx) => match L::THIS_LEVEL {
                0 => (idx * GRANULE_SIZE) / L0_TABLE_ENTRY_SIZE_RANGE,
                1 => ((idx * GRANULE_SIZE) % L0_TABLE_ENTRY_SIZE_RANGE) / GRANULE_SIZE,
                _ => panic!(),
            },
            Err(_) => panic!(),
        }
    }

    fn map<F: FnOnce(usize) -> Result<(), Error>>(
        &mut self,
        index: usize,
        _level: usize,
        func: F,
    ) -> Result<(), Error> {
        map_l1_table(index, func)
    }

    fn get_inner(&self) -> Option<Self::Inner> {
        Some(self.0.lock().clone())
    }

    fn points_to_table_or_page(&self) -> bool {
        true
    }
}
