#![no_main]

use armv9a::regs::*;
use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::{Run, REC_ENTRY_FLAG_EMUL_MMIO};
use islet_rmm::rmi::{REC_ENTER, SUCCESS};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

const DataAbort: usize = 2 << 4;

#[derive(Debug, arbitrary::Arbitrary)]
struct DataAbortFuzz {
    esr: u64,
    hpfar: u64,
    far: u64,
}

fuzz_target!(|data: DataAbortFuzz| {
    let rd = mock::host::realm_setup();

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let esr = EsrEl2::new(data.esr)
        .set_masked_value(EsrEl2::SSE, 0) /* SSE triggers an unimplemented! */
        .set_masked_value(EsrEl2::EC, ESR_EL2_EC_DATA_ABORT)
        .get() as u64;

    unsafe {
        let rec = &mut *(rec1 as *mut Rec<'_>);
        rec.context.sys_regs.esr_el2 = esr;
    }

    let _ret = rmi::<REC_ENTER>(&[
        rec1,
        run1,
        REC_ENTER_EXIT_CMD,
        DataAbort,
        esr as usize,
        data.hpfar as usize,
        data.far as usize,
    ]);

    unsafe {
        let run = &mut *(run1 as *mut Run);
        run.set_entry_flags(REC_ENTRY_FLAG_EMUL_MMIO);
    }

    /* Complete data abort handling */
    let _ret = rmi::<REC_ENTER>(&[rec1, run1]);

    mock::host::realm_teardown(rd);
});
