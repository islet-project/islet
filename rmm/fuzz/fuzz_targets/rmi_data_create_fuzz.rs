#![no_main]

use islet_rmm::rmi::{DATA_CREATE, DATA_DESTROY, GRANULE_DELEGATE, GRANULE_UNDELEGATE,
                     RTT_INIT_RIPAS, RTT_READ_ENTRY, SUCCESS};
use islet_rmm::rmi::rtt_entry_state::{RMI_UNASSIGNED, RMI_ASSIGNED};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};


#[derive(Debug, arbitrary::Arbitrary)]
struct DataCreateFuzz {
    ipa: u64,
    flags: u64,
}

fuzz_target!(|data: DataCreateFuzz| -> Corpus {
    let ipa = data.ipa as usize;
    let flags = data.flags as usize;
    let base = (ipa / L3_SIZE) * L3_SIZE;
    let data_granule = alloc_granule(IDX_DATA1);
    let src = alloc_granule(IDX_SRC1);

    let top = match (base as usize).checked_add(L3_SIZE) {
        Some(x) => x,
        None => {
            return Corpus::Reject;
        }
    };

    let rd = realm_create();

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
    if (ret[0] != SUCCESS) {
        realm_destroy(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, ipa);

    let ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);
    if ret[0] != SUCCESS {
        mock::host::unmap(rd, ipa, false);
        realm_destroy(rd);
        return Corpus::Reject;
    }

    let _ret = rmi::<GRANULE_DELEGATE>(&[data_granule]);

    let ret = rmi::<DATA_CREATE>(&[rd, data_granule, ipa, src, flags]);

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
