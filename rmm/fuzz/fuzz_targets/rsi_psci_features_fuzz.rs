#![no_main]

use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{REC_ENTER, SUCCESS};
use islet_rmm::rsi::PSCI_FEATURES;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: u32| {
    let rd = mock::host::realm_setup();
    let feature = data as usize;

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let rsi_call = RecEnterFuzzCall {
        cmd: PSCI_FEATURES,
        args: &[feature],
    };

    let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

    let _ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

    mock::host::realm_teardown(rd);
});
