#![no_main]

use islet_rmm::rmi::{RTT_INIT_RIPAS, RTT_READ_ENTRY, SUCCESS};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

#[derive(Debug, arbitrary::Arbitrary)]
struct RTTInitRipasFuzz {
    base: u64,
    top: u64,
}

fuzz_target!(|data: RTTInitRipasFuzz| -> Corpus {
    let rd = realm_create();
    let base = data.base as usize;
    let top = data.top as usize;

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL]);
    if ret[0] != SUCCESS {
        realm_destroy(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, base);

    let _ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);

    mock::host::unmap(rd, base, false);

    realm_destroy(rd);
    Corpus::Keep
});
