use islet_rmm::granule::array::{GRANULE_REGION, GRANULE_SIZE};
use islet_rmm::granule::entry::GranuleGpt;

extern "C" {
    fn CPROVER_havoc_object(address: usize);
}

#[macro_export]
// DIFF: `islet_rmm` is used to find the names outside the `islet_rmm` crate
//       [before]
//       use crate::granule::array::{GRANULE_STATUS_TABLE, GRANULE_STATUS_TABLE_SIZE};
//       [after]
//       use islet_rmm::granule::array::{GRANULE_STATUS_TABLE, GRANULE_STATUS_TABLE_SIZE};
macro_rules! get_granule {
    ($addr:expr) => {{
        use islet_rmm::granule::array::{GRANULE_STATUS_TABLE, GRANULE_STATUS_TABLE_SIZE};
        use islet_rmm::granule::{granule_addr_to_index, validate_addr};
        use islet_rmm::rmi::error::Error;
        if !validate_addr($addr) {
            Err(Error::RmiErrorInput)
        } else {
            let idx = granule_addr_to_index($addr);
            if idx >= GRANULE_STATUS_TABLE_SIZE {
                Err(Error::RmiErrorInput)
            } else {
                let gst = &GRANULE_STATUS_TABLE;
                match gst.entries[idx].lock() {
                    Ok(guard) => Ok(guard),
                    Err(e) => Err(e),
                }
            }
        }
    }};
}

pub fn addr_is_granule_aligned(addr: usize) -> bool {
    addr % GRANULE_SIZE == 0
}

// This should be exclusively used by pre-condition
// to retrieve a granule's state value.
// `unwrap()` is not called to avoid a panic condition.
pub fn pre_granule_state(addr: usize) -> u8 {
    let gran_state_res = get_granule!(addr).map(|guard| guard.state());
    let gran_state = if let Ok(state) = gran_state_res {
        state
    } else {
        kani::any()
    };
    gran_state
}

// This should be exclusively used by post-condition
// to retrieve a granule's state value.
// `unwrap()` is guaranteed not to be reached.
pub fn post_granule_state(addr: usize) -> u8 {
    get_granule!(addr).map(|guard| guard.state()).unwrap()
}

// This should be exclusively used by pre-condition
// to retrieve a granule's gpt value.
// `unwrap()` is not called to avoid a panic condition.
pub fn pre_granule_gpt(addr: usize) -> GranuleGpt {
    let gran_gpt_res = get_granule!(addr).map(|guard| guard.gpt);
    let gran_gpt = if let Ok(gpt) = gran_gpt_res {
        gpt
    } else {
        kani::any()
    };
    gran_gpt
}

// This should be exclusively used by post-condition
// to retrieve a granule's gpt value.
// `unwrap()` is guaranteed not to be reached.
pub fn post_granule_gpt(addr: usize) -> GranuleGpt {
    get_granule!(addr).map(|guard| guard.gpt).unwrap()
}

pub fn initialize() {
    unsafe {
        CPROVER_havoc_object(GRANULE_REGION.as_ptr() as usize);
    }
}
