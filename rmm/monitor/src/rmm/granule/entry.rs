use crate::mm::address::PhysAddr;
use crate::mm::error::Error;
use crate::mm::page_table::{self, Level};

use super::translation::{addr_to_idx, L0_TABLE_ENTRY_SIZE_RANGE};
use super::{GranuleState, GRANULE_SIZE};
use spinning_top::Spinlock;

// Safety: concurrency safety
//  - For a granule status table that manages granules, it doesn't use a big lock for efficiency. So, we need to associate "lock" with each granule entry.
//  - For granule entries and a page table for them, removing a table or an entry is not supported on purpose to make things easier for concurrency safety.
//  - There are two points that requires locking,
//    - (1) subtable creation: `set_with_page_table_flags_via_alloc()` is in charge of it. In this function, validity_check-memory_alloc-set_state should be done under the lock.
//    - (2) state change on entry: `set()` should be done under the lock.
//  - Each entry can be either table (L1 table) or granule. This is determined by `Inner.table`.

pub struct Granule {
    state: u64,
    addr: usize,
}

impl Granule {
    fn set_state(&mut self, addr: usize, state: u64) -> Result<(), Error> {
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

        self.state = state;
        self.addr = addr;

        // According to the spec, transition from P to Delegated must get contents to be wiped.
        // TODO: when we try to zeroize() in the transition of "Undelegated->Delegated", it will face a tf-a-test failure. we need to look into this issue.
        // Note: currently, as we don't map RTT, rule out the case where prev == RTT.
        if prev != GranuleState::Undelegated
            && prev != GranuleState::RTT
            && state == GranuleState::Delegated
        {
            self.zeroize();
        }
        Ok(())
    }

    fn set_addr(&mut self, addr: usize) {
        self.addr = addr;
    }

    fn addr(&self) -> usize {
        self.addr
    }

    fn zeroize(&self) {
        let buf = self.addr;
        unsafe {
            core::ptr::write_bytes(buf as *mut usize, 0x0, GRANULE_SIZE / 8);
        }
    }
}

struct Inner {
    granule: Granule,
    table: bool,
    valid: bool,
}

impl Inner {
    fn new() -> Self {
        Self {
            granule: Granule {
                state: GranuleState::Undelegated,
                addr: 0,
            },
            table: false,
            valid: false,
        }
    }

    fn addr(&self) -> usize {
        self.granule.addr()
    }

    fn set_state(&mut self, addr: PhysAddr, state: u64) -> Result<(), Error> {
        self.granule.set_state(addr.as_usize(), state)?;
        self.table = false;
        self.valid = true;
        Ok(())
    }

    fn set_state_for_table(&mut self, addr: PhysAddr) -> Result<(), Error> {
        self.granule.set_addr(addr.as_usize());
        self.table = true;
        self.valid = true;
        Ok(())
    }

    fn valid(&self) -> bool {
        self.valid
    }
}

pub struct Entry(Spinlock<Inner>);
impl page_table::Entry for Entry {
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

    fn set_with_page_table_flags_via_alloc<T: FnMut() -> usize>(
        &mut self,
        mut alloc: T,
    ) -> Result<(), Error>
    where
        T: FnMut() -> usize,
    {
        let mut inner = self.0.lock();
        if !inner.valid() {
            let table = alloc();
            inner.set_state_for_table(PhysAddr::from(table))
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

    fn points_to_table_or_page(&self) -> bool {
        true
    }
}
