#![no_main]

use islet_rmm::rmi::rec::run::{Run, NR_GIC_LRS, NR_GPRS};
use islet_rmm::rmi::{REC_ENTER, SUCCESS};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
struct RunEntryFuzz {
    flags: u64,
    gprs: [u64; NR_GPRS],
    gicv3_hcr: u64,
    gicv3_lrs: [u64; NR_GIC_LRS],
}

fuzz_target!(|data: RunEntryFuzz| {
    let rd = mock::host::realm_setup();

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    unsafe {
        let run = &mut *(run1 as *mut Run);

        run.set_entry_flags(data.flags);
        run.set_entry_gic_hcr(data.gicv3_hcr);
        run.set_entry_gic_lrs(&data.gicv3_lrs, NR_GIC_LRS);

        for idx in 0..NR_GPRS {
            run.set_entry_gpr(idx, data.gprs[idx]).unwrap();
        }
    }

    let _ret = rmi::<REC_ENTER>(&[rec1, run1]);

    mock::host::realm_teardown(rd);
});
