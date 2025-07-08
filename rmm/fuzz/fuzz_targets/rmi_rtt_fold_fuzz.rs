#![no_main]

use islet_rmm::granule::GRANULE_SIZE;
use islet_rmm::rmi::{
    DATA_CREATE_UNKNOWN, DATA_DESTROY, GRANULE_DELEGATE, GRANULE_UNDELEGATE, RTT_CREATE, RTT_FOLD,
    RTT_INIT_RIPAS, RTT_MAP_UNPROTECTED, RTT_READ_ENTRY, RTT_UNMAP_UNPROTECTED, SUCCESS,
};
use islet_rmm::test_utils::{mock, *};
use mock::host::alloc_granule_l2_aligned;

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

const L2_PAGE_COUNT: usize = L2_SIZE / L3_SIZE;

#[derive(Debug, Copy, Clone, arbitrary::Arbitrary)]
enum FoldType {
    Unassigned,
    Assigned,
    NonHomogenous,
}

#[derive(Debug, arbitrary::Arbitrary)]
struct RTTFoldFuzz {
    base: u64,
    fold_type: FoldType,
    ram: bool,
}

/* In the cases of ASSIGNED_RAM and ASSIGNED_NS, before we can revert them
 * to their UNASSIGNED STATES, we need to unfold them first.
 */
fn unfold(rd: usize, base: usize) {
    let rtt_l3 = alloc_granule(IDX_RTT_LEVEL3);

    let ret = rmi::<RTT_CREATE>(&[rd, rtt_l3, base, MAP_LEVEL]);
    assert_eq!(ret[0], SUCCESS);
}

fn destroy_fold(rd: usize, base: usize, fold_type: FoldType, fold_success: bool) {
    let ns: bool = (base & (1 << IPA_WIDTH - 1)) != 0;

    match fold_type {
        FoldType::Assigned => {
            if fold_success {
                unfold(rd, base);
            }

            for idx in 0..L2_PAGE_COUNT {
                if ns {
                    let ret = rmi::<RTT_UNMAP_UNPROTECTED>(&[rd, base + idx * L3_SIZE, MAP_LEVEL]);
                    assert_eq!(ret[0], SUCCESS);
                } else {
                    let ret = rmi::<DATA_DESTROY>(&[rd, base + idx * L3_SIZE]);
                    assert_eq!(ret[0], SUCCESS);

                    let data_granule = alloc_granule_l2_aligned(IDX_L2_ALIGNED_DATA + idx);
                    let ret = rmi::<GRANULE_UNDELEGATE>(&[data_granule]);
                    assert_eq!(ret[0], SUCCESS);
                }
            }
        }
        FoldType::Unassigned => {}
        FoldType::NonHomogenous => {
            if ns {
                let ret = rmi::<RTT_UNMAP_UNPROTECTED>(&[rd, base, MAP_LEVEL]);
                assert_eq!(ret[0], SUCCESS);
            } else {
                let ret = rmi::<DATA_DESTROY>(&[rd, base]);
                assert_eq!(ret[0], SUCCESS);

                let data_granule = alloc_granule_l2_aligned(IDX_L2_ALIGNED_DATA);
                let ret = rmi::<GRANULE_UNDELEGATE>(&[data_granule]);
                assert_eq!(ret[0], SUCCESS);
            }
        }
    }
}

fn setup_fold(rd: usize, base: usize, fold_type: FoldType, ram: bool) {
    let top = base + L2_SIZE;
    let ns = (base & (1 << IPA_WIDTH - 1)) != 0;

    if ram && !ns {
        let ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);
        assert_eq!(ret[0], SUCCESS);
    }

    match fold_type {
        FoldType::Assigned => {
            for idx in 0..L2_PAGE_COUNT {
                if ns {
                    let ns_desc = alloc_granule_l2_aligned(IDX_L2_ALIGNED_DATA + idx);

                    let ret =
                        rmi::<RTT_MAP_UNPROTECTED>(&[rd, base + idx * L3_SIZE, MAP_LEVEL, ns_desc]);
                    assert_eq!(ret[0], SUCCESS);
                } else {
                    let data_granule = alloc_granule_l2_aligned(IDX_L2_ALIGNED_DATA + idx);
                    let ret = rmi::<GRANULE_DELEGATE>(&[data_granule]);
                    assert_eq!(ret[0], SUCCESS);

                    let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data_granule, base + idx * L3_SIZE]);
                    assert_eq!(ret[0], SUCCESS);
                }
            }
        }
        FoldType::Unassigned => {
            if ns {
                for idx in 0..L2_PAGE_COUNT {
                    let ns_desc = alloc_granule(IDX_NS_DESC);

                    let ret =
                        rmi::<RTT_MAP_UNPROTECTED>(&[rd, base + idx * L3_SIZE, MAP_LEVEL, ns_desc]);
                    assert_eq!(ret[0], SUCCESS);

                    let ret = rmi::<RTT_UNMAP_UNPROTECTED>(&[rd, base + idx * L3_SIZE, MAP_LEVEL]);
                    assert_eq!(ret[0], SUCCESS);
                }
            }
        }
        FoldType::NonHomogenous => {
            if ns {
                let ns_desc = alloc_granule_l2_aligned(IDX_L2_ALIGNED_DATA);

                let ret = rmi::<RTT_MAP_UNPROTECTED>(&[rd, base, MAP_LEVEL, ns_desc]);
                assert_eq!(ret[0], SUCCESS);
            } else {
                let data_granule = alloc_granule_l2_aligned(IDX_L2_ALIGNED_DATA);
                let ret = rmi::<GRANULE_DELEGATE>(&[data_granule]);
                assert_eq!(ret[0], SUCCESS);

                let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data_granule, base]);
                assert_eq!(ret[0], SUCCESS);
            }
        }
    }
}

fuzz_target!(|data: RTTFoldFuzz| -> Corpus {
    let base = (data.base as usize / L2_SIZE) * L2_SIZE;
    let top = match (data.base as usize).checked_add(L2_SIZE) {
        Some(x) => x,
        None => {
            return Corpus::Reject;
        }
    };
    let fold_type = data.fold_type;
    let ram = data.ram;

    let rd = realm_create();

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL]);
    if ret[0] != SUCCESS {
        realm_destroy(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, base);

    setup_fold(rd, base, fold_type, ram);

    let ret = rmi::<RTT_FOLD>(&[rd, base, MAP_LEVEL]);

    if ret[0] == SUCCESS {
        destroy_fold(rd, base, fold_type, true);

        match fold_type {
            FoldType::Unassigned => mock::host::unmap(rd, base, true),
            _ => mock::host::unmap(rd, base, false),
        }
    } else {
        destroy_fold(rd, base, fold_type, false);
        mock::host::unmap(rd, base, false);
    }

    realm_destroy(rd);
    Corpus::Keep
});
