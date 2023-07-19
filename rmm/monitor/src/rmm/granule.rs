use crate::rmm::PageMap;
extern crate alloc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spinning_top::Spinlock;

pub const GRANULE_SIZE: usize = 4096;
const SUB_TABLE_SIZE: usize = 1024 * 1024 * 8; // 8mb

pub const RET_SUCCESS: usize = 0;
pub const RET_STATE_ERR: usize = 1;
pub const RET_INVALID_ADDR: usize = 2;

pub trait GranuleMemoryMap {
    fn addr_to_idx(&self, phys: usize) -> Result<usize, ()>;
    fn max_granules(&self) -> usize;
}

struct GranuleStatusSubTable {
    active: AtomicBool,
    granules: Option<Vec<Spinlock<Granule>>>,
}

struct GranuleStatusTable {
    num_granules: usize,
    #[allow(dead_code)]
    num_sub_tables: usize,
    memory: &'static dyn GranuleMemoryMap,
    sub_tables: Vec<GranuleStatusSubTable>,
}

impl GranuleStatusSubTable {
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

    pub fn set_granule(&mut self, addr: usize, idx: usize, state: GranuleState, mm: PageMap) -> usize {
        if let Some(granules) = &mut self.granules {
            if let Some(granule) = granules.get_mut(idx) {
                let mut granule = granule.lock();
                let prev_state = granule.state;
                if !Self::validate_state(prev_state, state) {
                    return RET_STATE_ERR;
                }
                if granule.addr == 0 {
                    granule.addr = addr;
                }

                match state {
                    GranuleState::Delegated => {
                        if prev_state != GranuleState::Undelegated
                            && prev_state != GranuleState::Delegated
                            && prev_state != GranuleState::RTT
                        {
                            granule.zeroize();
                            mm.unmap(granule.addr);
                        }
                        granule.set_state(state);
                    },
                    GranuleState::Undelegated => granule.set_state(state),
                    GranuleState::RTT => granule.set_state(state),
                    _ => {
                        granule.set_state(state);
                        mm.map(granule.addr, true);
                    },
                }
                return RET_SUCCESS;
            }
        }
        return RET_INVALID_ADDR;
    }
}

impl GranuleStatusTable {
    pub fn new(memory: &'static dyn GranuleMemoryMap) -> Self {
        let max_granules = memory.max_granules();
        let num_sub_tables = (max_granules * GRANULE_SIZE) / SUB_TABLE_SIZE;
        let num_granules = SUB_TABLE_SIZE / GRANULE_SIZE;
        let mut sub_tables = Vec::<GranuleStatusSubTable>::new();

        info!("[JB] num_granules: {}, num_sub_tables: {}, max_granules: {}", num_granules, num_sub_tables, max_granules);

        for _ in 0..num_sub_tables {
            sub_tables.push(GranuleStatusSubTable {
                active: AtomicBool::new(false),
                granules: None,
            });
        }
        Self {
            num_granules,
            num_sub_tables,
            memory,
            sub_tables,
        }
    }

    pub fn set_granule(&mut self, addr: usize, state: GranuleState, mm: PageMap) -> usize {
        let idx = if let Ok(v) = self.memory.addr_to_idx(addr) {
            v
        } else {
            return RET_INVALID_ADDR;
        };
        let table_idx = (idx * GRANULE_SIZE) / SUB_TABLE_SIZE;
        let sub_table_idx = ((idx * GRANULE_SIZE) % SUB_TABLE_SIZE) / GRANULE_SIZE;

        if let Some(sub_table) = self.sub_tables.get_mut(table_idx) {
            let active = sub_table.active.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
            match active {
                Ok(_) => {
                    // create a sub table and update a granule
                    let mut granules = Vec::<Spinlock<Granule>>::new();
                    for _ in 0..self.num_granules {
                        granules.push(Spinlock::new(
                            Granule {
                                state: GranuleState::Undelegated,
                                addr: 0,
                            }
                        ));
                    }
                    sub_table.granules = Some(granules);
                    return sub_table.set_granule(addr, sub_table_idx, state, mm);
                },
                Err(_) => {
                    // update a granule
                    // TODO: we need AtomicOption (or another AtomicBool) and waits for getting AtomicOption (i.e., contention in creating a sub table)
                    return sub_table.set_granule(addr, sub_table_idx, state, mm);
                },
            }
        }

        return RET_INVALID_ADDR;
    }
}

static mut GST: Option<GranuleStatusTable> = None;

pub fn create_gst(memory: &'static dyn GranuleMemoryMap) {
    unsafe {
        if GST.is_none() {
            GST = Some(GranuleStatusTable::new(memory));
        }
    };
}

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

impl Default for GranuleState {
    fn default() -> Self {
        Self::Undelegated
    }
}

impl Granule {
    #[allow(dead_code)]
    fn get_state(&self) -> GranuleState {
        self.state
    }
    fn set_state(&mut self, state: GranuleState) {
        self.state = state;
    }
    #[allow(dead_code)]
    fn set_addr(&mut self, addr: usize) {
        self.addr = addr;
    }
    #[allow(dead_code)]
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
    if let Some(ref mut gst) = unsafe { &mut GST } {
        gst.set_granule(addr, state, mm)
    } else {
        RET_INVALID_ADDR
    }
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
