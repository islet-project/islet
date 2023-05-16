use crate::rmm::PageMap;
extern crate alloc;
use alloc::collections::btree_map::BTreeMap;

use spinning_top::{Spinlock, SpinlockGuard};

const FVP_DRAM0_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8000_0000,
    end: 0x8000_0000 + 0x7C00_0000 - 1,
};
const FVP_DRAM1_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8_8000_0000,
    end: 0x8_8000_0000 + 0x8000_0000 - 1,
};

const GRANULE_SIZE: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GranuleState {
    Undelegated,
    Delegated,
    RD,
    Rec,
    RecAux,
    Data,
    RTT,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Granule {
    state: GranuleState,
    addr: usize,
}

static mut GRANULE: Spinlock<BTreeMap<usize, Granule>> = Spinlock::new(BTreeMap::new());

impl Default for GranuleState {
    fn default() -> Self {
        Self::Undelegated
    }
}

// Define a trait RmmGranule.
pub trait RmmGranule {
    fn get_state(&self) -> GranuleState;
    fn set_state(&mut self, state: GranuleState);
    fn set_addr(&mut self, addr: usize);
    fn addr(&self) -> usize;
    fn zeroize(&self);
}

// Implement RmmGranule trait for Granule struct.
impl RmmGranule for Granule {
    fn get_state(&self) -> GranuleState {
        self.state
    }
    fn set_state(&mut self, state: GranuleState) {
        self.state = state;
    }
    fn set_addr(&mut self, addr: usize) {
        self.addr = addr;
    }
    fn addr(&self) -> usize {
        return self.addr;
    }
    fn zeroize(&self) {
        let buf = self.addr;
        unsafe {
            core::ptr::write_bytes(buf as *mut usize, 0x0, GRANULE_SIZE / 8);
        }
    }
}

// Implement set_granule for Granule state control and check the valid.
pub fn set_granule(addr: usize, state: GranuleState, mm: PageMap) {
    let mut granule = find_granule(addr);
    let prev_state = granule.get_mut(&addr).unwrap().get_state();
    match state {
        GranuleState::Delegated => {
            if prev_state != GranuleState::Undelegated
                && prev_state != GranuleState::Delegated
                && prev_state != GranuleState::RTT
            {
                granule.get_mut(&addr).unwrap().zeroize();
                mm.unmap(addr);
            }
            granule.get_mut(&addr).unwrap().set_state(state);
        }
        GranuleState::RTT => granule.get_mut(&addr).unwrap().set_state(state),
        GranuleState::Undelegated => {
            // TODO: after RIPAS implement, it will be changed to panic
            if prev_state != GranuleState::Delegated {
                warn!(
                    "granule[{:X}]? expected:Delegated->found:{:?}",
                    addr, prev_state
                );
            }
            granule.remove(&addr);
        }
        _ => {
            // TODO: after RIPAS implement, it will be changed to panic
            if prev_state != GranuleState::Delegated {
                warn!(
                    "granule[{:X}]? expected:Delegated->found:{:?}",
                    addr, prev_state
                );
            }
            granule.get_mut(&addr).unwrap().set_state(state);
            mm.map(addr, true);
        }
    }
}

// Define a function find_granule which returns a Granule instance from the GRANULE array using the given address and expected state.
fn find_granule(addr: usize) -> SpinlockGuard<'static, BTreeMap<usize, Granule>> {
    validate_addr(addr);
    unsafe {
        let mut gr = GRANULE.lock();
        match gr.get_mut(&addr) {
            Some(_) => gr,
            None => {
                let new = Granule {
                    state: GranuleState::Undelegated,
                    addr,
                };
                gr.insert(addr, new);
                gr
            }
        }
    }
}

// Define a function check_validate_addr which check the address is valid.
fn validate_addr(addr: usize) {
    if addr % GRANULE_SIZE != 0 {
        // if the address is not aligned.
        warn!("address need to align 0x{:X}", addr);
    }
    if !(FVP_DRAM0_REGION.contains(&addr) || FVP_DRAM1_REGION.contains(&addr)) {
        // if the address is out of range.
        panic!("address is strange 0x{:X}", addr);
    }
}

#[cfg(test)]
mod test {
    use crate::rmm::granule;
    use crate::rmm::granule::GranuleState;
    use crate::rmm::granule::RmmGranule;

    const TEST_ADDR: usize = 0x880c_0000;
    const TEST_WRONG_ADDR: usize = 0x7900_0000;

    #[test]
    fn test_add_granule() {
        let mut g = granule::find_granule(TEST_ADDR);
        assert!(g.get_mut(&TEST_ADDR).unwrap().get_state() == GranuleState::Undelegated);
    }

    #[test]
    fn test_find_granule_with_addr() {
        let mut g = granule::find_granule(TEST_ADDR);
        g.get_mut(&TEST_ADDR)
            .unwrap()
            .set_state(GranuleState::Delegated);

        assert!(g.get_mut(&TEST_ADDR).unwrap().get_state() == GranuleState::Delegated);
    }

    #[test]
    #[should_panic]
    fn test_find_granule_with_wrong_addr() {
        let _g = granule::find_granule(TEST_WRONG_ADDR);
    }

    #[test]
    fn test_get_addr() {
        let mut g = granule::find_granule(TEST_ADDR);
        g.get_mut(&TEST_ADDR)
            .unwrap()
            .set_state(GranuleState::Delegated);

        assert!(g.get_mut(&TEST_ADDR).unwrap().addr() == TEST_ADDR);
    }

    #[test]
    fn test_addr() {
        let mut g = granule::find_granule(TEST_ADDR);
        g.get_mut(&TEST_ADDR)
            .unwrap()
            .set_state(GranuleState::Delegated);

        assert!(g.get_mut(&TEST_ADDR).unwrap().addr() == TEST_ADDR);
    }
}
