#![no_main]

use islet_rmm::rmi::FEATURES;
use islet_rmm::test_utils::*;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: u64| {
    let feature_reg_index = data as usize;
    let _ret = rmi::<FEATURES>(&[feature_reg_index]);
});
