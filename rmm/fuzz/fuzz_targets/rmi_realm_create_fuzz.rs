#![no_main]

use islet_rmm::rmi::{REALM_CREATE, REALM_DESTROY, REALM_ACTIVATE, GRANULE_DELEGATE, GRANULE_UNDELEGATE, SUCCESS};
use islet_rmm::granule::GRANULE_STATUS_TABLE_SIZE;
use islet_rmm::rmi::realm::params::Params as RealmParams;
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
struct RealmParamsFuzz {
    flags: u64,
    s2sz: u8,
    sve_vl: u8,
    num_bps: u8,
    num_wps: u8,
    pmu_num_ctrs: u8,
    hash_algo: u8,
    rpv: [u8; 64],
    vmid: u16,
    rtt_level_start: i64,
    rtt_num_start: u32
}

fuzz_target!(|data: RealmParamsFuzz| {
    let (rd, rtt, params_ptr) = (
        alloc_granule(IDX_RD),
        alloc_granule(IDX_RTT_LEVEL0),
        alloc_granule(IDX_REALM_PARAMS),
    );

    let _ret = rmi::<GRANULE_DELEGATE>(&[rd]);
    let _ret = rmi::<GRANULE_DELEGATE>(&[rtt]);

    unsafe {
        let params = &mut *(params_ptr as *mut RealmParams);

        params.flags = data.flags;
        params.s2sz = data.s2sz;
        params.sve_vl = data.sve_vl;
        params.num_bps = data.num_bps;
        params.num_wps = data.num_wps;
        params.pmu_num_ctrs = data.pmu_num_ctrs;
        params.hash_algo = data.hash_algo;
        params.rpv = data.rpv;
        params.vmid = data.vmid;
        params.rtt_base = rtt as u64;
        params.rtt_level_start = data.rtt_level_start;
        params.rtt_num_start = data.rtt_num_start;
    }

    let ret = rmi::<REALM_CREATE>(&[rd, params_ptr]);

    if ret[0] == SUCCESS {
        let _ret = rmi::<REALM_ACTIVATE>(&[rd]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<REALM_DESTROY>(&[rd]);
        assert_eq!(ret[0], SUCCESS);
    }

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[rd]);
    let _ret = rmi::<GRANULE_UNDELEGATE>(&[rtt]);
});
