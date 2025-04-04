#![no_main]

use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{
    DATA_CREATE_UNKNOWN, DATA_DESTROY, GRANULE_DELEGATE, GRANULE_UNDELEGATE, REALM_ACTIVATE,
    REC_ENTER, RTT_INIT_RIPAS, RTT_READ_ENTRY, SUCCESS,
};
use islet_rmm::rsi::HOST_CALL;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{fuzz_target, Corpus};

fuzz_target!(|data: u64| -> Corpus {
    let ipa = (data as usize) / L3_SIZE * L3_SIZE;
    let top = match (ipa as usize).checked_add(L3_SIZE) {
        Some(x) => x,
        None => {
            return Corpus::Reject;
        }
    };

    let rd = mock::host::realm_unactivated_setup();

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let data_granule = alloc_granule(IDX_DATA1);
    let mut fuzz_ret = Corpus::Keep;

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
    if ret[0] != SUCCESS {
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, ipa);

    let ret = rmi::<RTT_INIT_RIPAS>(&[rd, ipa, top]);
    if ret[0] != SUCCESS {
        mock::host::unmap(rd, ipa, false);
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    let ret = rmi::<REALM_ACTIVATE>(&[rd]);
    assert_eq!(ret[0], SUCCESS);

    let _ret = rmi::<GRANULE_DELEGATE>(&[data_granule]);

    let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data_granule, ipa]);
    if ret[0] == SUCCESS {
        let rsi_call = RecEnterFuzzCall {
            cmd: HOST_CALL,
            args: &[ipa],
        };

        let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

        /* Enter first time to run HOST_CALL */
        let _ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

        /* Enter again to finish host call */
        let _ret = rmi::<REC_ENTER>(&[rec1, run1]);

        let ret = rmi::<DATA_DESTROY>(&[rd, ipa]);
        assert_eq!(ret[0], SUCCESS);
    } else {
        fuzz_ret = Corpus::Reject;
    }

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[data_granule]);

    mock::host::unmap(rd, ipa, false);

    mock::host::realm_teardown(rd);

    fuzz_ret
});
