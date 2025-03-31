#![no_main]

use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{
    DATA_CREATE_UNKNOWN, DATA_DESTROY, GRANULE_DELEGATE, GRANULE_UNDELEGATE, REC_ENTER,
    RTT_INIT_RIPAS, RTT_READ_ENTRY, SUCCESS,
};
use islet_rmm::rsi::IPA_STATE_GET;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

#[derive(Debug, arbitrary::Arbitrary)]
struct GetRipasFuzz {
    base: u64,
    top: u64,
}

fuzz_target!(|data: GetRipasFuzz| -> Corpus {
    let rd = mock::host::realm_setup();
    let base = data.base as usize;
    let top = data.top as usize;

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let data_granule = alloc_granule(IDX_DATA1);

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL]);
    if ret[0] != SUCCESS {
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, base);

    let rsi_call = RecEnterFuzzCall {
        cmd: IPA_STATE_GET,
        args: &[base, top],
    };

    let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

    let _ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[data_granule]);

    mock::host::unmap(rd, base, false);

    mock::host::realm_teardown(rd);

    Corpus::Keep
});
