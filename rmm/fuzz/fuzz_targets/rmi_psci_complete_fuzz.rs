#![no_main]

use islet_rmm::rmi::{REC_ENTER, PSCI_COMPLETE, SUCCESS};
use islet_rmm::rec::Rec;
use islet_rmm::rec::context::set_reg;
use islet_rmm::rsi::{PSCI_CPU_ON, PSCI_AFFINITY_INFO};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
enum PSCICommandFuzz {
    PSCI_CPU_ON,
    PSCI_AFFINITY_INFO,
}

#[derive(Debug, arbitrary::Arbitrary)]
struct PSCICompleteFuzz {
    status: usize,
    psci_command: PSCICommandFuzz,
    target_runnable: bool,
}

fuzz_target!(|data: PSCICompleteFuzz| {
    let rd = mock::host::realm_setup();
    let status = data.status;
    let psci_command = data.psci_command;
    let target_runnable = data.target_runnable as u64;

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let _ret = rmi::<REC_ENTER>(&[rec1, run1]);

    let rec2 = alloc_granule(IDX_REC2);

    unsafe {
        let rec = &mut *(rec1 as *mut Rec<'_>);
        let target_rec = &mut *(rec2 as *mut Rec<'_>);

        match psci_command {
            PSCICommandFuzz::PSCI_CPU_ON => { set_reg(rec, 0, PSCI_CPU_ON).unwrap() },
            PSCICommandFuzz::PSCI_AFFINITY_INFO => { set_reg(rec, 0, PSCI_AFFINITY_INFO).unwrap() },
        }

        target_rec.set_runnable(target_runnable);
    }

    let _ret = rmi::<PSCI_COMPLETE>(&[rec1, rec2, status]);

    mock::host::realm_teardown(rd);
});
