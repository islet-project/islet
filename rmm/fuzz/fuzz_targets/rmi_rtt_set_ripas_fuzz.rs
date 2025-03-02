#![no_main]

use islet_rmm::rmi::{RTT_SET_RIPAS, REC_ENTER, RTT_READ_ENTRY, SUCCESS};
use islet_rmm::rec::Rec;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

#[derive(Debug, arbitrary::Arbitrary)]
enum RIPASValue {
    EMPTY = 0,
    RAM = 1,
}

#[derive(Debug, arbitrary::Arbitrary)]
struct RTTSetRIPASFuzz {
    base: u64,
    top: u64,

    ripas_state: RIPASValue,
    ripas_flags: u64,
}

fuzz_target!(|data: RTTSetRIPASFuzz| -> Corpus {
    let rd = mock::host::realm_setup();
    let base = data.base as usize;
    let top = data.top as usize;
    let ripas_state = data.ripas_state as u8;
    let ripas_flags = data.ripas_flags as u64;

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL]);
    if (ret[0] != SUCCESS) {
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let _ret = rmi::<REC_ENTER>(&[rec1, run1]);

    unsafe {
        let rec = &mut *(rec1 as *mut Rec<'_>);

        rec.set_ripas(data.base, data.top, ripas_state, ripas_flags);
    }

    mock::host::map(rd, base);

    let _ret = rmi::<RTT_SET_RIPAS>(&[rd, rec1, base, top]);

    mock::host::unmap(rd, base, false);

    mock::host::realm_teardown(rd);
    Corpus::Keep
});
