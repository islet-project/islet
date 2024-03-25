use vmsa::address::PhysAddr;
use vmsa::error::Error;
use vmsa::guard::EntryGuard;
use vmsa::page_table::{self, Level};

use super::translation::{add_l1_table, addr_to_idx, get_l1_table_addr, L0_TABLE_ENTRY_SIZE_RANGE};
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
        // check if this state transition is valid
        let prev = self.state;
        let valid = match prev {
            GranuleState::Undelegated => {
                state == GranuleState::Delegated || state == GranuleState::Undelegated
            }
            GranuleState::Delegated => state != GranuleState::Delegated,
            _ => state == GranuleState::Delegated,
        };
        if !valid {
            error!(
                "Granule state transition failed: prev[{:?}] -> next[{:?}]",
                prev, state
            );
            return Err(Error::MmStateError);
        }

        // check if it needs to be wiped out
        match state {
            GranuleState::Delegated => {
                // transition from something to Delegatated means "destroyed". (e.g., Rd --> Delegated when REALM_DESTROY)
                // so, it releases its parent and check its refcount to determine whether it's safe to get destroyed.
                // Note: currently, as we don't map RTT, rule out the case where prev == RTT.
                if prev != GranuleState::RTT {
                    self.parent.take();
                    destroy_callback()?;
                    self.zeroize();
                }
            }
            GranuleState::Undelegated => {
                if prev == GranuleState::Delegated {
                    self.zeroize();
                }
            }
            _ => {}
        }

        self.addr = addr;
        self.state = state;
        Ok(())
    }

    fn set_parent(&mut self, parent: Inner) -> Result<(), Error> {
        // parent-child state validation check
        // (Parent, Child): (Rd, Rec) --> only one case at this moment.
        if self.state() != GranuleState::Rec || parent.granule.state() != GranuleState::RD {
            return Err(Error::MmWrongParentChild);
        }
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

    #[cfg(not(test))]
    fn zeroize(&mut self) {
        let buf = self.addr;
        unsafe {
            core::ptr::write_bytes(buf as *mut usize, 0x0, GRANULE_SIZE / 8);
        }
    }

    #[cfg(test)]
    fn zeroize(&mut self) {}
}

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

    pub fn addr(&self) -> usize {
        self.granule.addr()
    }

    pub fn state(&self) -> u64 {
        self.granule.state()
    }

    pub fn set_state(&mut self, addr: PhysAddr, state: u64) -> Result<(), Error> {
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

    pub fn set_parent(&mut self, parent: Inner) -> Result<(), Error> {
        Rc::get_mut(&mut self.granule)
            .map_or_else(|| Err(Error::MmRefcountError), |g| g.set_parent(parent))
    }

    pub fn check_parent(&self, parent: &Inner) -> Result<(), Error> {
        if let Some(src_parent) = &self.granule.parent {
            if core::ptr::eq(src_parent, parent) {
                Ok(())
            } else {
                Err(Error::MmStateError)
            }
        } else {
            Err(Error::MmStateError)
        }
    }

    pub fn num_children(&self) -> usize {
        Rc::strong_count(&self.granule) - 1
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

    fn pte(&self) -> u64 {
        todo!();
    }

    fn mut_pte(&mut self) -> &mut Self::Inner {
        self.0.get_mut()
    }

    fn address(&self, _level: usize) -> Option<PhysAddr> {
        Some(PhysAddr::from(self.0.lock().addr()))
    }

    fn set(&mut self, addr: PhysAddr, flags: u64, _is_raw: bool) -> Result<(), Error> {
        self.0.lock().set_state(addr, flags)
    }

    fn set_with_page_table_flags(&mut self, _addr: PhysAddr) -> Result<(), Error> {
        // Note: this function is not used. To enable entry-level locking,
        // it needs to be forced to use `set_with_page_table_flags_via_alloc()`.
        Err(Error::MmErrorOthers)
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

    fn subtable(&self, index: usize, _level: usize) -> Result<usize, Error> {
        get_l1_table_addr(index)
    }

    fn lock(&self) -> Result<Option<EntryGuard<'_, Self::Inner>>, Error> {
        let inner = self.0.lock();
        let addr = inner.addr();
        let state = inner.state();
        let valid = inner.valid();

        if !valid {
            Err(Error::MmStateError)
        } else {
            Ok(Some(EntryGuard::new(inner, addr, state)))
        }
    }

    fn points_to_table_or_page(&self) -> bool {
        true
    }
}
