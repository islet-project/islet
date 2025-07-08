#![no_main]

use islet_rmm::granule::GRANULE_STATUS_TABLE_SIZE;
use islet_rmm::rmi::{GRANULE_UNDELEGATE, SUCCESS};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
enum GranuleAddress {
    GranuleRegion(u16),
    RandomAddress(u64),
}

fuzz_target!(|data: GranuleAddress| {
    let addr: usize = match data {
        GranuleAddress::GranuleRegion(idx) => {
            alloc_granule((idx as usize) % GRANULE_STATUS_TABLE_SIZE)
        }
        GranuleAddress::RandomAddress(addr) => addr as usize,
    };

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[addr]);
});
