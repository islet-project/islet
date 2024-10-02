use crate::granule::array::GRANULE_STATUS_TABLE;
use crate::rmi::error::Error;

use super::{GranuleState, GRANULE_SIZE};
use safe_abstraction::raw_ptr;
use spinning_top::{Spinlock, SpinlockGuard};
use vmsa::guard::Content;

#[cfg(not(any(kani, miri, test)))]
use crate::granule::{FVP_DRAM0_REGION, FVP_DRAM1_IDX, FVP_DRAM1_REGION};

// Safety: concurrency safety
//  - For a granule status table that manages granules, it doesn't use a big lock for efficiency.
//    So, we need to associate "lock" with each granule entry.

#[cfg(not(any(kani, miri, test)))]
pub struct Granule {
    /// granule state
    state: u8,
}
#[cfg(any(kani, miri, test))]
// DIFF: `gpt` ghost field is added to track GPT entry's status
pub struct Granule {
    /// granule state
    state: u8,
    /// granule protection table (ghost field)
    pub gpt: GranuleGpt,
}

#[cfg(kani)]
#[derive(Copy, Clone, PartialEq, kani::Arbitrary)]
pub enum GranuleGpt {
    GPT_NS,
    GPT_OTHER,
    GPT_REALM,
}

#[cfg(any(miri, test))]
#[derive(Copy, Clone, PartialEq)]
pub enum GranuleGpt {
    GPT_NS,
    GPT_OTHER,
    GPT_REALM,
}

impl Granule {
    #[cfg(not(any(kani, miri, test)))]
    fn new() -> Self {
        let state = GranuleState::Undelegated;
        Granule { state }
    }
    #[cfg(any(kani, miri, test))]
    // DIFF: `state` and `gpt` are filled with non-deterministic values
    fn new() -> Self {
        #[cfg(kani)]
        {
            let state = kani::any();
            kani::assume(state >= GranuleState::Undelegated && state <= GranuleState::RTT);
            let gpt = {
                if state != GranuleState::Undelegated {
                    GranuleGpt::GPT_REALM
                } else {
                    let gpt = kani::any();
                    kani::assume(gpt != GranuleGpt::GPT_REALM);
                    gpt
                }
            };
            Granule { state, gpt }
        }

        #[cfg(any(miri, test))]
        {
            let state = GranuleState::Undelegated;
            Self {
                state: GranuleState::Undelegated,
                gpt: GranuleGpt::GPT_NS,
            }
        }
    }

    #[cfg(any(kani, miri, test))]
    pub fn set_gpt(&mut self, gpt: GranuleGpt) {
        self.gpt = gpt;
    }

    #[cfg(any(kani, miri, test))]
    pub fn is_valid(&self) -> bool {
        self.state >= GranuleState::Undelegated &&
        self.state <= GranuleState::RTT &&
        // XXX: the below condition holds from beta0 to eac4
        if self.state != GranuleState::Undelegated {
            self.gpt == GranuleGpt::GPT_REALM
        } else {
            self.gpt != GranuleGpt::GPT_REALM
        }
    }

    pub fn state(&self) -> u8 {
        self.state
    }

    pub fn set_state(&mut self, state: u8) -> Result<(), Error> {
        let prev = self.state;
        if (prev == GranuleState::Delegated && state == GranuleState::Undelegated)
            || (state == GranuleState::Delegated)
        {
            self.zeroize();
        }
        self.state = state;
        Ok(())
    }

    pub fn content_mut<T>(&mut self) -> Result<raw_ptr::SafetyAssumed<T>, Error>
    where
        T: Content + raw_ptr::SafetyChecked + raw_ptr::SafetyAssured,
    {
        let addr = self.index_to_addr();
        Ok(raw_ptr::assume_safe::<T>(addr)?)
    }

    pub fn new_uninit_with<T>(&mut self, value: T) -> Result<raw_ptr::SafetyAssumed<T>, Error>
    where
        T: Content + raw_ptr::SafetyChecked + raw_ptr::SafetyAssured,
    {
        let addr = self.index_to_addr();
        Ok(raw_ptr::assume_safe_uninit_with::<T>(addr, value)?)
    }

    pub fn content<T>(&self) -> Result<raw_ptr::SafetyAssumed<T>, Error>
    where
        T: Content + raw_ptr::SafetyChecked + raw_ptr::SafetyAssured,
    {
        let addr = self.index_to_addr();
        Ok(raw_ptr::assume_safe::<T>(addr)?)
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
        let index = (entry_addr - table_base) / core::mem::size_of::<Entry>();
        index
    }

    #[cfg(not(any(kani, miri, test)))]
    fn index_to_addr(&self) -> usize {
        let idx = self.index();
        if idx < FVP_DRAM1_IDX {
            return FVP_DRAM0_REGION.start + (idx * GRANULE_SIZE);
        }
        FVP_DRAM1_REGION.start + ((idx - FVP_DRAM1_IDX) * GRANULE_SIZE)
    }
    #[cfg(any(kani, miri, test))]
    // DIFF: calculate addr using GRANULE_REGION
    pub fn index_to_addr(&self) -> usize {
        use crate::granule::{GRANULE_REGION, GRANULE_STATUS_TABLE_SIZE};
        let idx = self.index();
        assert!(idx >= 0 && idx < GRANULE_STATUS_TABLE_SIZE);

        #[cfg(any(miri, test))]
        return crate::test_utils::align_up(unsafe {
            GRANULE_REGION.as_ptr() as usize + (idx * GRANULE_SIZE)
        });

        #[cfg(kani)]
        return unsafe { GRANULE_REGION.as_ptr() as usize + (idx * GRANULE_SIZE) };
    }

    #[cfg(not(any(kani, miri, test)))]
    fn zeroize(&mut self) {
        let addr = self.index_to_addr();

        // Safety: This operation writes to a Granule outside the RMM Memory region,
        //         thus not violating RMM's Memory Safety.
        //         (ref. RMM Specification A2.2.4 Granule Wiping)
        unsafe {
            core::ptr::write_bytes(addr as *mut u8, 0x0, GRANULE_SIZE);
        }
    }
    #[cfg(any(kani, miri, test))]
    // DIFF: assertion is added to reduce the proof burden
    //       `write_bytes()` uses a small count value
    fn zeroize(&mut self) {
        let addr = self.index_to_addr();
        let g_start = unsafe { crate::granule::array::GRANULE_REGION.as_ptr() as usize };
        let g_end = g_start + crate::granule::array::GRANULE_MEM_SIZE;
        assert!(addr >= g_start && addr < g_end);

        unsafe {
            core::ptr::write_bytes(addr as *mut u8, 0x0, 8);
            assert!(*(addr as *const u8) == 0);
        }
    }
}

pub struct Entry(Spinlock<Granule>);
impl Entry {
    #[cfg(not(any(kani, miri, test)))]
    pub fn new() -> Self {
        Self(Spinlock::new(Granule::new()))
    }
    #[cfg(any(kani, miri, test))]
    // DIFF: assertion is added to reduce the proof burden
    pub fn new() -> Self {
        let granule = Granule::new();
        assert!(granule.is_valid());
        Self(Spinlock::new(granule))
    }

    pub fn lock(&self) -> Result<SpinlockGuard<'_, Granule>, Error> {
        let granule = self.0.lock();
        Ok(granule)
    }
}
