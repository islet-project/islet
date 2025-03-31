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

    ripas_state: u8,
    ripas_flags: u64,
    is_assigned: bool,
}

fuzz_target!(|data: RTTSetRIPASFuzz| -> Corpus {
    let base = (data.base as usize) / L3_SIZE * L3_SIZE;
    let top = match (base as usize).checked_add(L3_SIZE) {
        Some(x) => x,
        None => {
            return Corpus::Reject;
        }
    };

    let rd = mock::host::realm_unactivated_setup();
    let ripas_state = data.ripas_state as usize;
    let ripas_flags = data.ripas_flags as usize;
    let is_assigned = data.is_assigned;

    let mut fuzz_ret = Corpus::Keep;

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL]);
    if (ret[0] != SUCCESS) {
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, base);

    /* Only ASSIGNED_RAM entries can be converted to DESTROYED */
    let ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);
    if ret[0] != SUCCESS {
        mock::host::unmap(rd, base, false);
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    let data_granule = alloc_granule(IDX_DATA1);
    let _ret = rmi::<GRANULE_DELEGATE>(&[data_granule]);

    let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data_granule, base]);

    if ret[0] == SUCCESS {
        let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

        let _ret = rmi::<REALM_ACTIVATE>(&[rd]);
        assert_eq!(ret[0], SUCCESS);

        /* Create DESTROYED entries */
        let ret = rmi::<DATA_DESTROY>(&[rd, base]);
        assert_eq!(ret[0], SUCCESS);

        /* Create ASSIGNED_DESTROYED entries, otherwise UNASSIGNED_DESTROYED */
        if is_assigned {
            let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data_granule, base]);
            assert_eq!(ret[0], SUCCESS);
        }

        let rsi_call = RecEnterFuzzCall {
            cmd: IPA_STATE_SET,
            args: &[base, top, ripas_state, ripas_flags],
        };

        let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

        let ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

        if ret[0] == SUCCESS {
            let ret = rmi::<RTT_SET_RIPAS>(&[rd, rec1, base, top]);

            /* Complete RIPAS change */
            let _ret = rmi::<REC_ENTER>(&[rec1, run1]);
        }

        if is_assigned {
            let ret = rmi::<DATA_DESTROY>(&[rd, base]);
            assert_eq!(ret[0], SUCCESS);
        }
    } else {
        fuzz_ret = Corpus::Reject
    }

    mock::host::unmap(rd, base, false);

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[data_granule]);

    mock::host::realm_teardown(rd);
    fuzz_ret
});
