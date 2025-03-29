use crate::event::Context;
use crate::event::Mainloop;
use crate::granule::GRANULE_SIZE;
use crate::monitor::Monitor;
use crate::rmi::realm::Params as RealmParams;
use crate::rmi::rec::params::Params as RecParams;
use crate::rmi::*;

pub use mock::host::{alloc_granule, granule_addr};

use alloc::vec::Vec;

pub const REC_ENTER_EXIT_CMD: usize = 0;

pub struct RecEnterFuzzCall<'a> {
    pub cmd: usize,
    pub args: &'a [usize],
}

pub fn rmi<const COMMAND: usize>(arg: &[usize]) -> Vec<usize> {
    let monitor = Monitor::new();

    let mut ctx = Context::new(COMMAND);
    ctx.init_arg(&arg);
    ctx.init_ret(&[0; 8]);

    let handler = monitor.rmi.on_event.get(&COMMAND).unwrap();
    if let Err(code) = handler(&ctx.arg, &mut ctx.ret, &monitor) {
        ctx.ret[0] = code.into();
    }
    ctx.ret.to_vec()
}

pub fn extract_bits(value: usize, start: u32, end: u32) -> usize {
    let num_bits = end - start + 1;
    let mask = if num_bits == usize::BITS {
        usize::MAX
    } else {
        (1 << num_bits) - 1
    };
    (value >> start) & mask
}

pub fn realm_create() -> usize {
    for mocking_addr in &[alloc_granule(IDX_RD), alloc_granule(IDX_RTT_LEVEL0)] {
        let ret = rmi::<GRANULE_DELEGATE>(&[*mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
    }

    let (rd, rtt, params_ptr) = (
        alloc_granule(IDX_RD),
        alloc_granule(IDX_RTT_LEVEL0),
        alloc_granule(IDX_REALM_PARAMS),
    );

    unsafe {
        let params = &mut *(params_ptr as *mut RealmParams);
        params.s2sz = 40;
        params.rtt_num_start = 1;
        params.rtt_level_start = 0;
        params.rtt_base = rtt as u64;
    };

    let ret = rmi::<REALM_CREATE>(&[rd, params_ptr]);
    assert_eq!(ret[0], SUCCESS);

    rd
}

pub fn realm_destroy(rd: usize) {
    let ret = rmi::<REALM_DESTROY>(&[rd]);
    assert_eq!(ret[0], SUCCESS);

    for mocking_addr in &[granule_addr(IDX_RD), granule_addr(IDX_RTT_LEVEL0)] {
        let ret = rmi::<GRANULE_UNDELEGATE>(&[*mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
    }
}

pub fn rec_create(rd: usize, idx_rec: usize, idx_rec_params: usize, idx_rec_aux_start: usize) {
    let (rec, params_ptr) = (alloc_granule(idx_rec), alloc_granule(idx_rec_params));
    let ret = rmi::<GRANULE_DELEGATE>(&[rec]);
    assert_eq!(ret[0], SUCCESS);

    let ret = rmi::<REC_AUX_COUNT>(&[rd]);
    assert_eq!(ret[0], SUCCESS);
    assert_eq!(ret[1], MAX_REC_AUX_GRANULES);

    let aux_count = ret[1];
    unsafe {
        let params = &mut *(params_ptr as *mut RecParams);
        params.pc = 0;
        params.flags = 1; // RMI_RUNNABLE
        params.mpidr = (idx_rec % IDX_REC1) as u64;
        params.num_aux = aux_count as u64;

        for idx in 0..aux_count {
            let mocking_addr = alloc_granule(idx + idx_rec_aux_start);
            let ret = rmi::<GRANULE_DELEGATE>(&[mocking_addr]);
            assert_eq!(ret[0], SUCCESS);

            params.aux[idx] = mocking_addr as u64;
        }
    }

    let ret = rmi::<REC_CREATE>(&[rd, rec, params_ptr]);
    assert_eq!(ret[0], SUCCESS);
}

pub fn rec_destroy(idx_rec: usize, idx_rec_aux_start: usize) {
    let rec = granule_addr(idx_rec);
    let ret = rmi::<REC_DESTROY>(&[rec]);
    assert_eq!(ret[0], SUCCESS);

    let ret = rmi::<GRANULE_UNDELEGATE>(&[rec]);
    assert_eq!(ret[0], SUCCESS);

    for idx in 0..MAX_REC_AUX_GRANULES {
        let mocking_addr = granule_addr(idx + idx_rec_aux_start);
        let ret = rmi::<GRANULE_UNDELEGATE>(&[mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
    }
}

pub fn data_create(rd: usize, ipa: usize, idx_data: usize, idx_src: usize) {
    const RMI_NO_MEASURE_CONTENT: usize = 0;

    mock::host::map(rd, ipa);

    let base = (ipa / L3_SIZE) * L3_SIZE;
    let top = base + L3_SIZE;
    let ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);
    assert_eq!(ret[0], SUCCESS);

    let data = alloc_granule(idx_data);
    let ret = rmi::<GRANULE_DELEGATE>(&[data]);
    assert_eq!(ret[0], SUCCESS);

    let src = alloc_granule(idx_src);
    let flags = RMI_NO_MEASURE_CONTENT;

    let ret = rmi::<DATA_CREATE>(&[rd, data, ipa, src, flags]);
    assert_eq!(ret[0], SUCCESS);
}

pub fn align_up(addr: usize) -> usize {
    let align_mask = GRANULE_SIZE - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        (addr | align_mask) + 1
    }
}

#[cfg(fuzzing)]
pub fn align_up_l2(addr: usize) -> usize {
    let align_mask = L2_SIZE - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        (addr | align_mask) + 1
    }
}

pub const IDX_RD: usize = 0;
pub const IDX_RTT_LEVEL0: usize = 1;
pub const IDX_REALM_PARAMS: usize = 2;
pub const IDX_RTT_LEVEL1: usize = 3;
pub const IDX_RTT_LEVEL2: usize = 4;
pub const IDX_RTT_LEVEL3: usize = 5;
pub const IDX_RTT_OTHER: usize = 6;

pub const IDX_REC1: usize = 7;
pub const IDX_REC2: usize = 8;
pub const IDX_REC1_AUX: usize = 9;
pub const IDX_REC2_AUX: usize = 25; // 9 + 16
pub const IDX_REC1_PARAMS: usize = 41; // 25 + 16
pub const IDX_REC2_PARAMS: usize = 42;
pub const IDX_REC1_RUN: usize = 43;

pub const IDX_NS_DESC: usize = 45;
pub const IDX_DATA1: usize = 46;
pub const IDX_DATA2: usize = 47;
pub const IDX_DATA3: usize = 48;
pub const IDX_DATA4: usize = 49;
pub const IDX_SRC1: usize = 50;
pub const IDX_SRC2: usize = 51;

#[cfg(fuzzing)]
pub const IDX_L2_ALIGNED_DATA: usize = 0;

pub const MAP_LEVEL: usize = 3;
pub const L3_SIZE: usize = GRANULE_SIZE;
pub const L2_SIZE: usize = 512 * L3_SIZE;
pub const L1_SIZE: usize = 512 * L2_SIZE;
pub const L0_SIZE: usize = 512 * L1_SIZE;
pub const IPA_WIDTH: usize = 40;
pub const ATTR_NORMAL_WB_WA_RA: usize = 1 << 2;
pub const ATTR_STAGE2_AP_RW: usize = 3 << 6;
pub const ATTR_INNER_SHARED: usize = 3 << 8;

/// This function is a temporary workaround to pass the MIRI test due to a memory leak bug
/// related to the RMM Page Table. It forces the RMM Page Table to drop at the end of the
/// test, preventing the memory leak issue from occurring during MIRI testing.
///
/// - Memory Leak in RMM Page Table: During the mapping/unmapping process, the Page Table
///   might not deallocate even when there are no entries in Level 1-3 tables.
///
/// Note: When testing this function individually, set `TEST_TOTAL` to 1.
pub fn miri_teardown() {
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering;

    const TEST_TOTAL: usize = 11;
    static TEST_COUNT: AtomicUsize = AtomicUsize::new(0);
    TEST_COUNT.fetch_add(1, Ordering::SeqCst);

    if TEST_COUNT.load(Ordering::SeqCst) == TEST_TOTAL {
        crate::mm::translation::drop_page_table();
    }
}

pub mod mock {
    pub mod host {
        use super::super::*;

        use crate::granule::{GRANULE_REGION, GRANULE_SIZE};
        use crate::rmi::{RTT_CREATE, RTT_DESTROY, RTT_READ_ENTRY};

        pub use alloc_granule as granule_addr;

        /// This function simulates memory allocation by using a portion of the
        /// pre-allocated memory within the RMM.
        /// It mocks the memory allocation provided by host and is designed
        /// to bypass provenance issues detected by MIRI.
        pub fn alloc_granule(idx: usize) -> usize {
            let start = unsafe { GRANULE_REGION.as_ptr() as usize };
            let first = crate::test_utils::align_up(start);
            first + idx * GRANULE_SIZE
        }

        /// Mock allocation of granules starting from an L2 aligned address for RTT fold fuzzing.
        /// The granule region is made big enough to make space for these granules in the
        /// fuzzing setup. It is also ensured these granules do not collide with the
        /// regular mock granules.
        #[cfg(fuzzing)]
        pub fn alloc_granule_l2_aligned(idx: usize) -> usize {
            let start = unsafe { GRANULE_REGION.as_ptr() as usize };
            let first = crate::test_utils::align_up_l2(start + L2_SIZE);
            first + idx * GRANULE_SIZE
        }

        pub fn map(rd: usize, ipa: usize) {
            let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
            assert_eq!(ret[0], SUCCESS);

            let (level, _state, _desc, _ripas) = (ret[1], ret[2], ret[3], ret[4]);
            let (rtt_l1, rtt_l2, rtt_l3) = (
                alloc_granule(IDX_RTT_LEVEL1),
                alloc_granule(IDX_RTT_LEVEL2),
                alloc_granule(IDX_RTT_LEVEL3),
            );

            if level < 1 {
                let ret = rmi::<GRANULE_DELEGATE>(&[rtt_l1]);
                assert_eq!(ret[0], SUCCESS);

                let ipa_aligned = (ipa / L0_SIZE) * L0_SIZE;
                let ret = rmi::<RTT_CREATE>(&[rd, rtt_l1, ipa_aligned, 1]);
                assert_eq!(ret[0], SUCCESS);
            }

            if level < 2 {
                let ret = rmi::<GRANULE_DELEGATE>(&[rtt_l2]);
                assert_eq!(ret[0], SUCCESS);

                let ipa_aligned = (ipa / L1_SIZE) * L1_SIZE;
                let ret = rmi::<RTT_CREATE>(&[rd, rtt_l2, ipa_aligned, 2]);
                assert_eq!(ret[0], SUCCESS);
            }

            if level < 3 {
                let ret = rmi::<GRANULE_DELEGATE>(&[rtt_l3]);
                assert_eq!(ret[0], SUCCESS);

                let ipa_aligned = (ipa / L2_SIZE) * L2_SIZE;
                let ret = rmi::<RTT_CREATE>(&[rd, rtt_l3, ipa_aligned, 3]);
                assert_eq!(ret[0], SUCCESS);
            }
        }

        pub fn unmap(rd: usize, ipa: usize, folded: bool) {
            if !folded {
                let ipa_aligned = (ipa / L2_SIZE) * L2_SIZE;
                let ret = rmi::<RTT_DESTROY>(&[rd, ipa_aligned, 3]);
                assert_eq!(ret[0], SUCCESS);
            }

            let ipa_aligned = (ipa / L1_SIZE) * L1_SIZE;
            let ret = rmi::<RTT_DESTROY>(&[rd, ipa_aligned, 2]);
            assert_eq!(ret[0], SUCCESS);

            let ipa_aligned = (ipa / L0_SIZE) * L0_SIZE;
            let ret = rmi::<RTT_DESTROY>(&[rd, ipa_aligned, 1]);
            assert_eq!(ret[0], SUCCESS);

            for mocking_addr in &[
                granule_addr(IDX_RTT_LEVEL1),
                granule_addr(IDX_RTT_LEVEL2),
                granule_addr(IDX_RTT_LEVEL3),
            ] {
                let ret = rmi::<GRANULE_UNDELEGATE>(&[*mocking_addr]);
                assert_eq!(ret[0], SUCCESS);
            }
        }

        pub fn realm_setup() -> usize {
            let rd = realm_create();
            rec_create(rd, IDX_REC1, IDX_REC1_PARAMS, IDX_REC1_AUX);
            rec_create(rd, IDX_REC2, IDX_REC2_PARAMS, IDX_REC2_AUX);

            let ret = rmi::<REALM_ACTIVATE>(&[rd]);
            assert_eq!(ret[0], SUCCESS);

            rd
        }

        pub fn realm_unactivated_setup() -> usize {
            let rd = realm_create();
            rec_create(rd, IDX_REC1, IDX_REC1_PARAMS, IDX_REC1_AUX);
            rec_create(rd, IDX_REC2, IDX_REC2_PARAMS, IDX_REC2_AUX);

            rd
        }

        pub fn realm_teardown(rd: usize) {
            rec_destroy(IDX_REC1, IDX_REC1_AUX);
            rec_destroy(IDX_REC2, IDX_REC2_AUX);
            realm_destroy(rd);
        }
    }

    pub mod realm {
        use super::super::*;
        use crate::event::realmexit::RecExitReason;
        use crate::rec::context::set_reg;
        use crate::rec::Rec;
        use crate::rmi::rec::run::Run;
        use crate::rsi::PSCI_CPU_ON;

        pub fn setup_psci_complete(rec: &mut Rec<'_>, run: &mut Run) {
            let reason: u64 = RecExitReason::PSCI.into();

            // caller
            rec.set_psci_pending(true);
            run.set_exit_reason(reason as u8);
            run.set_gpr(0, PSCI_CPU_ON as u64).unwrap();

            let target_mpidr = IDX_REC2 % IDX_REC1;
            set_reg(rec, 1, target_mpidr).unwrap();
        }

        pub fn setup_ripas_state(rec: &mut Rec<'_>, run: &mut Run) {
            let ipa_base: u64 = 0;
            let ipa_top: u64 = 0x1000;
            const RSI_RAM: u8 = 1;
            const RSI_NO_CHANGE_DESTROYED: u64 = 0;
            run.set_ripas(ipa_base, ipa_top, RSI_RAM);
            rec.set_ripas(ipa_base, ipa_top, RSI_RAM, RSI_NO_CHANGE_DESTROYED);
        }
    }
}
