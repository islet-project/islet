use super::Rec;
use crate::pmu::*;

use core::array::from_fn;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use aarch64_cpu::registers::*;
use armv9a::regs::pmu::*;
use armv9a::{MDCR_EL2, PMCR_EL0};

use crate::config::NUM_OF_CPU;
use crate::cpu::get_cpu_id;
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;

#[repr(C)]
#[derive(Default, Debug)]
pub struct PmuRegister {
    pub pmcr_el0: u64,
    pub pmccfiltr_el0: u64,
    pub pmccntr_el0: u64,
    pub pmcntenset_el0: u64,
    pub pmcntenclr_el0: u64,
    pub pmintenset_el1: u64,
    pub pmintenclr_el1: u64,
    pub pmovsset_el0: u64,
    pub pmovsclr_el0: u64,
    pub pmselr_el0: u64,
    pub pmuserenr_el0: u64,
    pub pmxevcntr_el0: u64,
    pub pmxevtyper_el0: u64,
    pub pmevcntr_el0: [u64; MAX_EVCNT],
    pub pmevtyper_el0: [u64; MAX_EVCNT],
}

lazy_static! {
    static ref NS_PMU: [Mutex<PmuRegister>; NUM_OF_CPU] =
        from_fn(|_| Mutex::new(PmuRegister::default()));
}

const CLEAR_MASK: u64 = 0x1_FFFF_FFFF;

pub fn init_pmu(rec: &mut Rec<'_>) {
    let (pmu_enabled, pmu_num_ctrs) = rec.pmu_config().expect("REASON");
    let mdcr_el2 = MDCR_EL2.get();
    let mask: u64 = MDCR_EL2::TPM::SET.value
        | MDCR_EL2::TPMCR::SET.value
        | (MDCR_EL2::HPMN.mask << MDCR_EL2::HPMN.shift);
    rec.context.mdcr_el2 = if pmu_enabled {
        (mdcr_el2 & !mask) | MDCR_EL2::HPMN.val(pmu_num_ctrs as u64).value
    } else {
        mdcr_el2 | (MDCR_EL2::TPM::SET + MDCR_EL2::TPMCR::SET).value
    };
    let rec_pmu = &mut rec.context.pmu;
    rec_pmu.pmcr_el0 = if pmu_enabled {
        (PMCR_EL0::LC::SET + PMCR_EL0::DP::SET + PMCR_EL0::C::SET + PMCR_EL0::P::SET).into()
    } else {
        (PMCR_EL0::LC::SET + PMCR_EL0::DP::SET).into()
    };
}

fn restore_pmu(pmu: &PmuRegister, num_cntrs: usize) {
    PMCR_EL0.set(pmu.pmcr_el0);
    PMCCFILTR_EL0.set(pmu.pmccfiltr_el0);
    PMCCNTR_EL0.set(pmu.pmccntr_el0);
    PMCNTENSET_EL0.set(pmu.pmcntenset_el0);
    PMCNTENCLR_EL0.set(pmu.pmcntenclr_el0 ^ CLEAR_MASK);
    PMINTENSET_EL1.set(pmu.pmintenset_el1);
    PMINTENCLR_EL1.set(pmu.pmintenclr_el1 ^ CLEAR_MASK);
    PMOVSSET_EL0.set(pmu.pmovsset_el0);
    PMOVSCLR_EL0.set(pmu.pmovsclr_el0 ^ CLEAR_MASK);
    PMSELR_EL0.set(pmu.pmselr_el0);
    PMUSERENR_EL0.set(pmu.pmuserenr_el0);
    PMXEVCNTR_EL0.set(pmu.pmxevcntr_el0);
    PMXEVTYPER_EL0.set(pmu.pmxevtyper_el0);
    set_pmev_regs(num_cntrs, &pmu.pmevcntr_el0, &pmu.pmevtyper_el0);
}

pub fn restore_state(rec: &Rec<'_>) {
    let (enabled, num_cntrs) = rec.pmu_config().expect("REASON");

    if !enabled {
        return;
    }
    MDCR_EL2.set(rec.context.mdcr_el2);
    let rec_pmu = &rec.context.pmu;
    restore_pmu(rec_pmu, num_cntrs);
}

fn save_pmu(pmu: &mut PmuRegister, num_cntrs: usize) {
    pmu.pmcr_el0 = PMCR_EL0.get();
    pmu.pmccfiltr_el0 = PMCCFILTR_EL0.get();
    pmu.pmccntr_el0 = PMCCNTR_EL0.get();
    pmu.pmcntenset_el0 = PMCNTENSET_EL0.get();
    pmu.pmcntenclr_el0 = PMCNTENCLR_EL0.get();
    pmu.pmintenset_el1 = PMINTENSET_EL1.get();
    pmu.pmintenclr_el1 = PMINTENCLR_EL1.get();
    pmu.pmovsset_el0 = PMOVSSET_EL0.get();
    pmu.pmovsclr_el0 = PMOVSCLR_EL0.get();
    pmu.pmselr_el0 = PMSELR_EL0.get();
    pmu.pmuserenr_el0 = PMUSERENR_EL0.get();
    pmu.pmxevcntr_el0 = PMXEVCNTR_EL0.get();
    pmu.pmxevtyper_el0 = PMXEVTYPER_EL0.get();
    get_pmev_regs(num_cntrs, &mut pmu.pmevcntr_el0, &mut pmu.pmevtyper_el0);
}

pub fn save_state(rec: &mut Rec<'_>) {
    let (enabled, num_cntrs) = rec.pmu_config().expect("REASON");

    if !enabled {
        return;
    }
    let rec_pmu = &mut rec.context.pmu;
    save_pmu(rec_pmu, num_cntrs);
}

pub fn save_host_state(rec: &Rec<'_>) {
    let (enabled, _) = rec.pmu_config().expect("REASON");
    if !enabled {
        return;
    }
    let mut ns_pmu = NS_PMU[get_cpu_id()].lock();
    // Event counter and cycle counter can be reset by P and C bits in PMCR_EL0
    // thus save and restore all counters.
    save_pmu(&mut ns_pmu, pmu_num_ctrs() as usize);
}

pub fn restore_host_state(rec: &Rec<'_>) {
    let (enabled, _) = rec.pmu_config().expect("REASON");
    if !enabled {
        return;
    }
    let ns_pmu = NS_PMU[get_cpu_id()].lock();
    // Event counter and cycle counter can be reset by P and C bits in PMCR_EL0
    // thus save and restore all counters.
    restore_pmu(&ns_pmu, pmu_num_ctrs() as usize);
}

pub fn pmu_overflow_active() -> bool {
    (PMOVSSET_EL0.get() & PMINTENSET_EL1.get() & PMCNTENSET_EL0.get()) != 0
        && PMCR_EL0.read(PMCR_EL0::E) != 0
}

pub fn send_state_to_host(_rec: &Rec<'_>, run: &mut Run) -> Result<(), Error> {
    run.set_pmu_overflow(pmu_overflow_active());
    Ok(())
}
