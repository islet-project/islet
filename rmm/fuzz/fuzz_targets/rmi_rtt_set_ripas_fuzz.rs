#![no_main]

use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{
    DATA_CREATE_UNKNOWN, DATA_DESTROY, GRANULE_DELEGATE, GRANULE_UNDELEGATE, REALM_ACTIVATE,
    REC_ENTER, RTT_INIT_RIPAS, RTT_READ_ENTRY, RTT_SET_RIPAS, SUCCESS,
};
use islet_rmm::rsi::IPA_STATE_SET;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

#[derive(Debug, arbitrary::Arbitrary)]
struct RTTSetRIPASFuzz {
    base: u64,
    top: u64,

    ripas_state: u8,
    ripas_flags: u64,
}

fuzz_target!(|data: RTTSetRIPASFuzz| -> Corpus {
    let rd = mock::host::realm_setup();
    let base = data.base as usize;
    let top = data.top as usize;
    let ripas_state = data.ripas_state as usize;
    let ripas_flags = data.ripas_flags as usize;

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL]);
    if (ret[0] != SUCCESS) {
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let rsi_call = RecEnterFuzzCall {
        cmd: IPA_STATE_SET,
        args: &[base, top, ripas_state, ripas_flags],
    };

    let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

    let ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

    if ret[0] == SUCCESS {
        mock::host::map(rd, base);

        let ret = rmi::<RTT_SET_RIPAS>(&[rd, rec1, base, top]);

        mock::host::unmap(rd, base, false);

        /* Complete RIPAS change */
        let _ret = rmi::<REC_ENTER>(&[rec1, run1]);
    }

    mock::host::realm_teardown(rd);
    Corpus::Keep
});
