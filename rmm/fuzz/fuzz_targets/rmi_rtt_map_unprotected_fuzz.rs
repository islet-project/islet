#![no_main]

use islet_rmm::rmi::rtt_entry_state::{RMI_ASSIGNED, RMI_UNASSIGNED};
use islet_rmm::rmi::{RTT_MAP_UNPROTECTED, RTT_READ_ENTRY, RTT_UNMAP_UNPROTECTED, SUCCESS};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

#[derive(Debug, arbitrary::Arbitrary)]
struct RTTUnprotected {
    ipa: u64,
    ATTR_NORMAL_WB_WA_RA: bool,
    ATTR_STAGE2_AP_RW: bool,
    ATTR_INNER_SHARED: bool,
}

fuzz_target!(|data: RTTUnprotected| -> Corpus {
    let rd = realm_create();
    let ipa = data.ipa as usize;
    let mut ns = alloc_granule(IDX_NS_DESC);

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
    if ret[0] != SUCCESS {
        realm_destroy(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, ipa);

    if data.ATTR_NORMAL_WB_WA_RA {
        ns = ns | ATTR_NORMAL_WB_WA_RA;
    }
    if data.ATTR_STAGE2_AP_RW {
        ns = ns | ATTR_STAGE2_AP_RW;
    }
    if data.ATTR_INNER_SHARED {
        ns = ns | ATTR_INNER_SHARED;
    }

    let ret = rmi::<RTT_MAP_UNPROTECTED>(&[rd, ipa, MAP_LEVEL, ns]);

    if ret[0] == SUCCESS {
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[2], RMI_ASSIGNED);

        let ret = rmi::<RTT_UNMAP_UNPROTECTED>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[2], RMI_UNASSIGNED);
    }

    mock::host::unmap(rd, ipa, false);

    realm_destroy(rd);
    Corpus::Keep
});
