use islet_rmm::granule::array::{GRANULE_REGION, GRANULE_SIZE};
use islet_rmm::granule::entry::GranuleGpt;
use islet_rmm::granule::validate_addr;
use islet_rmm::realm::rd::Rd;
use islet_rmm::realm::rd::State; // tmp
use islet_rmm::rec::Rec;
use islet_rmm::rec::State as RecState;

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

// TODO: find an object to which life should be bound
fn content_mut<T>(addr: usize) -> &'static mut T {
    unsafe { &mut *(addr as *mut T) }
}

// TODO: find an object to which life should be bound
fn content<T>(addr: usize) -> &'static T {
    unsafe { &*(addr as *const T) }
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

fn pre_valid_addr(addr: usize) -> usize {
    let indexed_addr = get_granule!(addr).map(|guard| guard.index_to_addr());
    let valid_addr = if let Ok(addr) = indexed_addr {
        addr
    } else {
        let addr = kani::any();
        kani::assume(validate_addr(addr));
        addr
    };
    valid_addr
}

pub fn pre_rd_state(addr: usize) -> State {
    let valid_addr = pre_valid_addr(addr);
    let rd = content_mut::<Rd>(valid_addr);
    rd.state()
}

pub fn post_rd_state(addr: usize) -> State {
    let valid_addr = get_granule!(addr)
        .map(|guard| guard.index_to_addr())
        .unwrap();
    let rd = content::<Rd>(valid_addr);
    rd.state()
}

pub fn pre_rec_state(addr: usize) -> RecState {
    let valid_addr = pre_valid_addr(addr);
    let rec = content::<Rec>(valid_addr);
    rec.get_state()
}

pub fn post_rec_aux_state(addr: usize) -> u8 {
    let valid_addr = pre_valid_addr(addr);
    let rec = content::<Rec>(valid_addr);
    // XXX: we currently check only the first entry to
    //      reduce the overall verification time
    let rec_aux = rec.aux(0) as usize;
    post_granule_state(rec_aux)
}

pub fn initialize() {
    unsafe {
        CPROVER_havoc_object(GRANULE_REGION.as_ptr() as usize);
    }
}
