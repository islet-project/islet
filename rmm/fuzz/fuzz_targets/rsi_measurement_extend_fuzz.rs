#![no_main]

use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{REC_ENTER, SUCCESS};
use islet_rmm::rsi::MEASUREMENT_EXTEND;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
struct MeasurementExtendFuzz {
    idx: u64,
    size: u64,
    values: [u64; 8],
}

fuzz_target!(|data: MeasurementExtendFuzz| {
    let rd = mock::host::realm_setup();
    let measurement_index = data.idx as usize;
    let size = data.size as usize;
    let values = &data.values;

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let args = &[
        measurement_index,
        size,
        values[0] as usize,
        values[1] as usize,
        values[2] as usize,
        values[3] as usize,
        values[4] as usize,
        values[5] as usize,
        values[6] as usize,
        values[7] as usize,
    ];

    let rsi_call = RecEnterFuzzCall {
        cmd: MEASUREMENT_EXTEND,
        args: args,
    };

    let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

    let _ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

    mock::host::realm_teardown(rd);
});
