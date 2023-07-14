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
pub const GRANULE_SIZE: usize = 4096;

pub const RET_SUCCESS: usize = 0;
pub const RET_STATE_ERR: usize = 1;
pub const RET_INVALID_ADDR: usize = 2;

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
pub fn set_granule(addr: usize, state: GranuleState, mm: PageMap) -> usize {
    if !validate_addr(addr) {
        return RET_INVALID_ADDR;
    }
    let mut granule = find_granule(addr);
    let prev_state = granule.get_mut(&addr).unwrap().get_state();
    if !validate_state(prev_state, state) {
        return RET_STATE_ERR;
    }
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
        GranuleState::Undelegated => {
            granule.remove(&addr);
        }
        GranuleState::RTT => granule.get_mut(&addr).unwrap().set_state(state),
        _ => {
            granule.get_mut(&addr).unwrap().set_state(state);
            mm.map(addr, true);
        }
    }
    RET_SUCCESS
}

// Define a function find_granule which returns a Granule instance from the GRANULE array using the given address and expected state.
fn find_granule(addr: usize) -> SpinlockGuard<'static, BTreeMap<usize, Granule>> {
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

fn validate_state(prev: GranuleState, cur: GranuleState) -> bool {
    if cur == GranuleState::Delegated && prev == cur {
        warn!("granule already delegated");
        return false;
    }
    if prev != GranuleState::Delegated && cur != GranuleState::Delegated {
        warn!("granule state err, prev:{:?}->to-be:{:?}", prev, cur);
        return false;
    }
    true
}

// Define a function check_validate_addr which check the address is valid.
fn validate_addr(addr: usize) -> bool {
    if addr % GRANULE_SIZE != 0 {
        // if the address is out of range.
        warn!("address need to be aligned 0x{:X}", addr);
        return false;
    }
    if !(FVP_DRAM0_REGION.contains(&addr) || FVP_DRAM1_REGION.contains(&addr)) {
        // if the address is out of range.
        warn!("address is strange 0x{:X}", addr);
        return false;
    }
    true
}

#[cfg(test)]
mod test {
    use crate::rmm::RmmPage;
    use crate::PageMap;

    use crate::rmm::granule;
    use crate::rmm::granule::GranuleState;

    const TEST_ADDR: usize = 0x880c_0000;
    const TEST_WRONG_ADDR: usize = 0x7900_0000;

    pub struct MockPageMap;
    impl MockPageMap {
        pub fn new() -> &'static MockPageMap {
            &MockPageMap {}
        }
    }
    impl RmmPage for MockPageMap {
        fn map(&self, _addr: usize, _secure: bool) -> bool {
            true
        }
        fn unmap(&self, _addr: usize) -> bool {
            true
        }
    }

    #[test]
    fn test_add_granule() {
        let dummy_map: PageMap = MockPageMap::new();
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Delegated, dummy_map)
                == granule::RET_SUCCESS
        );
        // restore state
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Undelegated, dummy_map)
                == granule::RET_SUCCESS
        );
    }

    #[test]
    fn test_find_granule_with_wrong_addr() {
        let dummy_map: PageMap = MockPageMap::new();
        assert!(
            granule::set_granule(TEST_WRONG_ADDR, GranuleState::Delegated, dummy_map)
                == granule::RET_INVALID_ADDR
        );
    }

    #[test]
    fn test_validate_state() {
        let dummy_map: PageMap = MockPageMap::new();
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Delegated, dummy_map)
                == granule::RET_SUCCESS
        );
        // RTT state don't use the map
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::RTT, dummy_map) == granule::RET_SUCCESS
        );
        // restore state
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Delegated, dummy_map)
                == granule::RET_SUCCESS
        );
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Undelegated, dummy_map)
                == granule::RET_SUCCESS
        );
    }

    #[test]
    fn test_validate_wrong_state() {
        let dummy_map: PageMap = MockPageMap::new();
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Delegated, dummy_map)
                == granule::RET_SUCCESS
        );
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Delegated, dummy_map)
                == granule::RET_STATE_ERR
        );

        // restore state
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Undelegated, dummy_map)
                == granule::RET_SUCCESS
        );
    }
}
