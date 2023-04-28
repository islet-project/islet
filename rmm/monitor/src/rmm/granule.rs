use crate::rmm::PageMap;
extern crate alloc;
use alloc::collections::btree_map::BTreeMap;

pub const FVP_DRAM0_BASE: usize = 0x8000_0000;
pub const FVP_DRAM0_SIZE: usize = 0x7C00_0000;
pub const FVP_DRAM0_END: usize = FVP_DRAM0_BASE + FVP_DRAM0_SIZE - 1;

const GRANULE_SIZE: usize = 4096;
const GRANULE_SHIFT: usize = 12;
const RMM_MAX_GRANULES: usize = 0x7_C000; // 507904(fvp dram0 size - ideal)
const GRANULE_BASE_ADDRESS: usize = FVP_DRAM0_BASE + 0x800_0000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GranuleState {
    Undelegated,
    Delegated,
    RD,
    Rec,
    RecAux,
    Data,
    RTT,
    Param,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Granule {
    state: GranuleState,
    // refcount: usize,
    idx: usize,
}

static mut GRANULE: BTreeMap<usize, Granule> = BTreeMap::new();

impl Default for GranuleState {
    fn default() -> Self {
        Self::Undelegated
    }
}

// Define a trait RmmGranule.
pub trait RmmGranule {
    fn get_state(&self) -> GranuleState;
    fn set_state(&mut self, state: GranuleState, mm: PageMap);
    fn get_index(&self) -> usize;
    fn set_index(&mut self, index: usize);
    // fn get_refcount(&self) -> usize;
    fn addr(&self) -> usize;
    fn zeroize(&self);
}

// Implement RmmGranule trait for Granule struct.
impl RmmGranule for Granule {
    // Implement get_state function for Granule which returns the state of the granule.
    fn get_state(&self) -> GranuleState {
        self.state
    }
    // Implement set_state function for Granule which sets the state of the granule and maps/unmaps the memory accordingly.
    fn set_state(&mut self, state: GranuleState, mm: PageMap) {
        match state {
            GranuleState::Delegated => {
                if self.state != GranuleState::Undelegated && self.state != GranuleState::Delegated
                {
                    mm.unmap(self.addr());
                }
                self.state = state;
            }
            GranuleState::RTT => info!("set state Data for Realm"),
            GranuleState::Undelegated => unsafe {
                if self.state == GranuleState::Param {
                    mm.unmap(self.addr());
                }
                self.state = state;
                // self.zeroize();
                GRANULE.remove(&self.idx);
            },
            GranuleState::Param => {
                if self.state != GranuleState::Undelegated {
                    warn!("check the granule state");
                }
                self.state = state;
                mm.map(self.addr(), false);
            }
            _ => {
                if self.state == GranuleState::Undelegated {
                    warn!("skip state Delegated");
                }
                self.state = state;
                mm.map(self.addr(), true);
            }
        }
    }
    // Implement get_index function for Granule which returns the index of the granule.
    fn get_index(&self) -> usize {
        self.idx
    }
    // Implement set_index function for Granule which sets the index of the granule.
    fn set_index(&mut self, index: usize) {
        self.idx = index;
    }
    // Implement get_refcount function for Granule which returns the reference count of the granule.
    // fn get_refcount(&self) -> usize {
    //     self.refcount
    // }
    // Implement addr function for Granule which returns the address of the granule.
    fn addr(&self) -> usize {
        return granule_idx_to_addr(self.idx);
    }

    fn zeroize(&self) {
        let buf = self.addr();
        for i in 0..GRANULE_SIZE {
            unsafe {
                core::ptr::write_volatile((buf + i) as *mut usize, 0);
            }
        }
    }
}

// Define a function find_granule which returns a Granule instance from the GRANULE array using the given address and expected state.
pub fn find_granule(addr: usize, expected_state: GranuleState) -> Option<&'static mut Granule> {
    let idx = granule_addr_to_idx(addr);
    if idx >= RMM_MAX_GRANULES {
        warn!("check the granule index {}", idx);
        return None;
    }

    unsafe {
        match GRANULE.get_mut(&idx) {
            Some(g) => {
                if expected_state != g.get_state() {
                    info!(
                        "check the {:X} granule state {:?}<-{:?}",
                        addr,
                        g.get_state(),
                        expected_state
                    );
                }
                Some(g)
            }
            None => {
                let new = Granule {
                    state: expected_state,
                    // refcount: 0,
                    idx: idx,
                };
                GRANULE.insert(idx, new);
                Some(GRANULE.get_mut(&idx).unwrap())
            }
        }
    }
}

// Define a function granule_addr_to_idx which returns the index of the granule using the given address.
fn granule_addr_to_idx(addr: usize) -> usize {
    if addr < GRANULE_BASE_ADDRESS || addr > FVP_DRAM0_END {
        // if the address is out of range.
        info!("address is strange 0x{:X}", addr);
        return usize::MAX;
    }
    (addr - GRANULE_BASE_ADDRESS) >> GRANULE_SHIFT
}

// Define a function granule_idx_to_addr which returns the address of the granule using the given index.
fn granule_idx_to_addr(idx: usize) -> usize {
    assert!(idx < RMM_MAX_GRANULES);
    GRANULE_BASE_ADDRESS + (idx << GRANULE_SHIFT)
}

#[cfg(test)]
mod test {
    use crate::rmm::granule;
    use crate::rmm::granule::GranuleState;
    use crate::rmm::granule::RmmGranule;
    use crate::rmm::PageMap;
    use crate::rmm::RmmPage;

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
        granule::find_granule(TEST_ADDR, GranuleState::Undelegated).expect("can't find granule");
        assert!(granule::find_granule(TEST_ADDR, GranuleState::Delegated) != None);
    }

    #[test]
    fn test_find_granule_with_addr() {
        let dummy_map: PageMap = MockPageMap::new();
        let g = granule::find_granule(TEST_ADDR, GranuleState::Undelegated)
            .expect("can't find granule");
        g.set_state(GranuleState::Delegated, dummy_map);

        assert!(granule::find_granule(TEST_ADDR, GranuleState::Delegated) != None);
    }

    #[test]
    fn test_find_granule_with_wrong_addr() {
        assert!(granule::find_granule(TEST_WRONG_ADDR, GranuleState::Undelegated) == None);
    }

    #[test]
    fn test_convert_addr() {
        let dummy_map: PageMap = MockPageMap::new();
        let g = granule::find_granule(TEST_ADDR, GranuleState::Undelegated)
            .expect("can't find granule");
        g.set_state(GranuleState::Delegated, dummy_map);

        let idx = granule::granule_addr_to_idx(TEST_ADDR);

        assert!(granule::granule_idx_to_addr(idx) == TEST_ADDR);
    }

    #[test]
    fn test_get_index() {
        let dummy_map: PageMap = MockPageMap::new();
        let g = granule::find_granule(TEST_ADDR, GranuleState::Undelegated)
            .expect("can't find granule");
        g.set_state(GranuleState::Delegated, dummy_map);

        let idx = granule::granule_addr_to_idx(TEST_ADDR);

        assert!(g.get_index() == idx);
    }

    #[test]
    fn test_addr() {
        let dummy_map: PageMap = MockPageMap::new();
        let g = granule::find_granule(TEST_ADDR, GranuleState::Undelegated)
            .expect("can't find granule");
        g.set_state(GranuleState::Delegated, dummy_map);

        assert!(g.addr() == TEST_ADDR);
    }
}
