#![no_main]

use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{PSCI_COMPLETE, REC_ENTER, SUCCESS, SUCCESS_REC_ENTER};
use islet_rmm::rsi::PSCI_AFFINITY_INFO;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
struct PSCIAffinityFuzz {
    target_affinity: u64,
    lowest_affinity_level: u32,
    status: usize,
    target_runnable: bool,
}

fuzz_target!(|data: PSCIAffinityFuzz| {
    let rd = mock::host::realm_setup();
    let target_affinity = data.target_affinity as usize;
    let lowest_affinity_level = data.lowest_affinity_level as usize;
    let status = data.status;
    let target_runnable = data.target_runnable as u64;

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let rec2 = alloc_granule(IDX_REC2);

    let rsi_call = RecEnterFuzzCall {
        cmd: PSCI_AFFINITY_INFO,
        args: &[target_affinity, lowest_affinity_level],
    };

    let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

    let ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

    if ret[0] == SUCCESS {
        unsafe {
            let target_rec = &mut *(rec2 as *mut Rec<'_>);
            target_rec.set_runnable(target_runnable);
        }

        let _ret = rmi::<PSCI_COMPLETE>(&[rec1, rec2, status]);
    }

    mock::host::realm_teardown(rd);
});
