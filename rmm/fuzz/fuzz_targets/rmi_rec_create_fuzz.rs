#![no_main]

use islet_rmm::rmi::rec::params::Params as RecParams;
use islet_rmm::rmi::rec::params::NR_GPRS;
use islet_rmm::rmi::{
    GRANULE_DELEGATE, GRANULE_UNDELEGATE, REC_AUX_COUNT, REC_CREATE, REC_DESTROY, SUCCESS,
};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
struct RecParamsFuzz {
    flags: u64,
    mpidr: u64,
    pc: u64,
    gprs: [u64; NR_GPRS],
}

fuzz_target!(|data: RecParamsFuzz| {
    let rd = realm_create();

    let ret = rmi::<REC_AUX_COUNT>(&[rd]);
    let rec_aux_count = ret[1];

    let (rec, params_ptr) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_PARAMS));

    let _ret = rmi::<GRANULE_DELEGATE>(&[rec]);

    unsafe {
        let params = &mut *(params_ptr as *mut RecParams);

        params.flags = data.flags;
        params.mpidr = data.mpidr;
        params.pc = data.pc;
        params.gprs = data.gprs;
        params.num_aux = rec_aux_count as u64;

        for idx in 0..rec_aux_count {
            let mocking_addr = alloc_granule(IDX_REC1_AUX + idx);
            let ret = rmi::<GRANULE_DELEGATE>(&[mocking_addr]);
            params.aux[idx] = mocking_addr as u64;
        }
    }

    let ret = rmi::<REC_CREATE>(&[rd, rec, params_ptr]);

    if ret[0] == SUCCESS {
        let ret = rmi::<REC_DESTROY>(&[rec]);
        assert_eq!(ret[0], SUCCESS);
    }

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[rec]);

    for idx in 0..rec_aux_count {
        let mocking_addr = alloc_granule(IDX_REC1_AUX + idx);
        let _ret = rmi::<GRANULE_UNDELEGATE>(&[rec]);
    }

    realm_destroy(rd);
});
