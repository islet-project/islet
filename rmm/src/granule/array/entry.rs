use crate::granule::array::GRANULE_STATUS_TABLE;
use crate::rmi::error::Error;

use super::{GranuleState, GRANULE_SIZE};
use spinning_top::{Spinlock, SpinlockGuard};
use vmsa::guard::Content;

use crate::granule::{FVP_DRAM0_REGION, FVP_DRAM1_IDX, FVP_DRAM1_REGION};

// Safety: concurrency safety
//  - For a granule status table that manages granules, it doesn't use a big lock for efficiency.
//    So, we need to associate "lock" with each granule entry.

pub struct Granule {
    /// granule state
    state: u8,
}

impl Granule {
    pub fn state(&self) -> u8 {
        self.state
    }

    pub fn set_state(&mut self, state: u8) -> Result<(), Error> {
        if state == GranuleState::Delegated {
            self.zeroize();
        }
        self.state = state;
        Ok(())
    }

    pub fn content_mut<T: Content>(&mut self) -> &mut T {
        let addr = self.index_to_addr();
        unsafe { &mut *(addr as *mut T) }
    }

    pub fn content<T: Content>(&self) -> &T {
        let addr = self.index_to_addr();
        unsafe { &*(addr as *const T) }
    }

    fn index(&self) -> usize {
        let entry_size = core::mem::size_of::<Entry>();
        let granule_size = core::mem::size_of::<Granule>();
        //  XXX: is there a clever way of getting the Entry from Granule (e.g., container_of())?
        //  [        Entry        ]
        //  [  offset ] [ Granule ]
        let granule_offset = entry_size - granule_size;
        let granule_addr = self as *const Granule as usize;
        let entry_addr = granule_addr - granule_offset;
        let gst = &GRANULE_STATUS_TABLE;
        let table_base = gst.entries.as_ptr() as usize;
        (entry_addr - table_base) / core::mem::size_of::<Entry>()
    }

    fn index_to_addr(&self) -> usize {
        let idx = self.index();
        if idx < FVP_DRAM1_IDX {
            return FVP_DRAM0_REGION.start + (idx * GRANULE_SIZE);
        }
        FVP_DRAM1_REGION.start + ((idx - FVP_DRAM1_IDX) * GRANULE_SIZE)
    }

    fn zeroize(&mut self) {
        let addr = self.index_to_addr();
        unsafe {
            core::ptr::write_bytes(addr as *mut u8, 0x0, GRANULE_SIZE);
        }
    }
}

pub struct Entry(Spinlock<Granule>);
impl Entry {
    pub fn new() -> Self {
        Self(Spinlock::new(Granule {
            state: GranuleState::Undelegated,
        }))
    }

    pub fn lock(&self) -> Result<SpinlockGuard<'_, Granule>, Error> {
        let granule = self.0.lock();
        Ok(granule)
    }
}
