#![no_main]

use islet_rmm::rmi::{GRANULE_DELEGATE, GRANULE_UNDELEGATE, RTT_CREATE, RTT_DESTROY, SUCCESS};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
struct RTTCreateFuzz {
    ipa: u64,
    level: i64,
}

fuzz_target!(|data: RTTCreateFuzz| {
    let rd = realm_create();
    let ipa = data.ipa as usize;
    let level = data.level as usize;

    let rtt = alloc_granule(IDX_RTT_LEVEL1);

    let _ret = rmi::<GRANULE_DELEGATE>(&[rtt]);

    let ret = rmi::<RTT_CREATE>(&[rd, rtt, ipa, level]);

    if ret[0] == SUCCESS {
        let ret = rmi::<RTT_DESTROY>(&[rd, ipa, level]);
        assert_eq!(ret[0], SUCCESS);
    }

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[rtt]);

    realm_destroy(rd);
});
