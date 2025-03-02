#![no_main]

use islet_rmm::rmi::{RTT_READ_ENTRY, SUCCESS};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
struct RTTEntryReadFuzz {
    ipa: u64,
    level: i64,
}

fuzz_target!(|data: RTTEntryReadFuzz| {
    let rd = realm_create();
    let ipa = data.ipa as usize;
    let level = data.level as usize;

    let _ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, level]);

    realm_destroy(rd);
});
