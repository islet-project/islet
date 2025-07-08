#![no_main]

use islet_rmm::rmi::VERSION;
use islet_rmm::test_utils::*;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: u64| {
    let req = data as usize;
    let _ret = rmi::<VERSION>(&[req]);
});
