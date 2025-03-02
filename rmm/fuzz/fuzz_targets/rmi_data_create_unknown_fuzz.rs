#![no_main]

use islet_rmm::rmi::{DATA_CREATE_UNKNOWN, DATA_DESTROY, GRANULE_DELEGATE,
                     GRANULE_UNDELEGATE, RTT_READ_ENTRY, SUCCESS};
use islet_rmm::rmi::rtt_entry_state::{RMI_UNASSIGNED, RMI_ASSIGNED};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

#[derive(Debug, arbitrary::Arbitrary)]
struct DataCreateFuzz {
    ipa: u64,
}

fuzz_target!(|data: DataCreateFuzz| -> Corpus {
    let rd = realm_create();
    let ipa = data.ipa as usize;
    let data_granule = alloc_granule(IDX_DATA1);

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
    if (ret[0] != SUCCESS) {
        realm_destroy(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, ipa);

    let _ret = rmi::<GRANULE_DELEGATE>(&[data_granule]);

    let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data_granule, ipa]);

    if ret[0] == SUCCESS {
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[2], RMI_ASSIGNED);

        let ret = rmi::<DATA_DESTROY>(&[rd, ipa]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[2], RMI_UNASSIGNED);
    }

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[data_granule]);

    mock::host::unmap(rd, ipa, false);
    realm_destroy(rd);

    Corpus::Keep
});
