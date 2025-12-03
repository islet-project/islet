use aarch64_cpu::registers::*;
use armv9a::regs::pmu::*;
use armv9a::PMCR_EL0;

pub const MAX_EVCNT: usize = 31;

#[allow(non_upper_case_globals)]
const FEAT_PMUv3p7: u64 = 7;
#[cfg(not(feature = "qemu"))]
const PMU_MIN_VER: u64 = FEAT_PMUv3p7;
#[cfg(feature = "qemu")]
const PMU_MIN_VER: u64 = 5; // FEAT_PMUv3p5

// ID_AA64DFR0_EL1
// HPMN0, bits [63:60] :
// Zero PMU event counters for a Guest operating system.
const HPMN0_MASK: u64 = 0xF << 60;

pub fn pmu_present() -> bool {
    trace!(
        "PMUVer: v3p{:?}",
        ID_AA64DFR0_EL1.read(ID_AA64DFR0_EL1::PMUVer)
    );
    ID_AA64DFR0_EL1.read(ID_AA64DFR0_EL1::PMUVer) >= PMU_MIN_VER
}

pub fn hpmn0_present() -> bool {
    trace!(
        "FEAT_HPMN0: {:?}",
        (ID_AA64DFR0_EL1.get() & HPMN0_MASK) >> 60
    );
    ID_AA64DFR0_EL1.get() & HPMN0_MASK != 0
}

pub fn pmu_num_ctrs() -> u64 {
    trace!("PMU # counters: {:?}", PMCR_EL0.read(PMCR_EL0::N));
    PMCR_EL0.read(PMCR_EL0::N)
}

fn store_pmev(n: usize, pmevcntr_el0: &mut [u64; MAX_EVCNT], pmevtyper_el0: &mut [u64; MAX_EVCNT]) {
    match n {
        0 => {
            pmevcntr_el0[0] = PMEVCNTR0_EL0.get();
            pmevtyper_el0[0] = PMEVTYPER0_EL0.get();
        }
        1 => {
            pmevcntr_el0[1] = PMEVCNTR1_EL0.get();
            pmevtyper_el0[1] = PMEVTYPER1_EL0.get();
        }
        2 => {
            pmevcntr_el0[2] = PMEVCNTR2_EL0.get();
            pmevtyper_el0[2] = PMEVTYPER2_EL0.get();
        }
        3 => {
            pmevcntr_el0[3] = PMEVCNTR3_EL0.get();
            pmevtyper_el0[3] = PMEVTYPER3_EL0.get();
        }
        4 => {
            pmevcntr_el0[4] = PMEVCNTR4_EL0.get();
            pmevtyper_el0[4] = PMEVTYPER4_EL0.get();
        }
        5 => {
            pmevcntr_el0[5] = PMEVCNTR5_EL0.get();
            pmevtyper_el0[5] = PMEVTYPER5_EL0.get();
        }
        6 => {
            pmevcntr_el0[6] = PMEVCNTR6_EL0.get();
            pmevtyper_el0[6] = PMEVTYPER6_EL0.get();
        }
        7 => {
            pmevcntr_el0[7] = PMEVCNTR7_EL0.get();
            pmevtyper_el0[7] = PMEVTYPER7_EL0.get();
        }
        8 => {
            pmevcntr_el0[8] = PMEVCNTR8_EL0.get();
            pmevtyper_el0[8] = PMEVTYPER8_EL0.get();
        }
        9 => {
            pmevcntr_el0[9] = PMEVCNTR9_EL0.get();
            pmevtyper_el0[9] = PMEVTYPER9_EL0.get();
        }
        10 => {
            pmevcntr_el0[10] = PMEVCNTR10_EL0.get();
            pmevtyper_el0[10] = PMEVTYPER10_EL0.get();
        }
        11 => {
            pmevcntr_el0[11] = PMEVCNTR11_EL0.get();
            pmevtyper_el0[11] = PMEVTYPER11_EL0.get();
        }
        12 => {
            pmevcntr_el0[12] = PMEVCNTR12_EL0.get();
            pmevtyper_el0[12] = PMEVTYPER12_EL0.get();
        }
        13 => {
            pmevcntr_el0[13] = PMEVCNTR13_EL0.get();
            pmevtyper_el0[13] = PMEVTYPER13_EL0.get();
        }
        14 => {
            pmevcntr_el0[14] = PMEVCNTR14_EL0.get();
            pmevtyper_el0[14] = PMEVTYPER14_EL0.get();
        }
        15 => {
            pmevcntr_el0[15] = PMEVCNTR15_EL0.get();
            pmevtyper_el0[15] = PMEVTYPER15_EL0.get();
        }
        16 => {
            pmevcntr_el0[16] = PMEVCNTR16_EL0.get();
            pmevtyper_el0[16] = PMEVTYPER16_EL0.get();
        }
        17 => {
            pmevcntr_el0[17] = PMEVCNTR17_EL0.get();
            pmevtyper_el0[17] = PMEVTYPER17_EL0.get();
        }
        18 => {
            pmevcntr_el0[18] = PMEVCNTR18_EL0.get();
            pmevtyper_el0[18] = PMEVTYPER18_EL0.get();
        }
        19 => {
            pmevcntr_el0[19] = PMEVCNTR19_EL0.get();
            pmevtyper_el0[19] = PMEVTYPER19_EL0.get();
        }
        20 => {
            pmevcntr_el0[20] = PMEVCNTR20_EL0.get();
            pmevtyper_el0[20] = PMEVTYPER20_EL0.get();
        }
        21 => {
            pmevcntr_el0[21] = PMEVCNTR21_EL0.get();
            pmevtyper_el0[21] = PMEVTYPER21_EL0.get();
        }
        22 => {
            pmevcntr_el0[22] = PMEVCNTR22_EL0.get();
            pmevtyper_el0[22] = PMEVTYPER22_EL0.get();
        }
        23 => {
            pmevcntr_el0[23] = PMEVCNTR23_EL0.get();
            pmevtyper_el0[23] = PMEVTYPER23_EL0.get();
        }
        24 => {
            pmevcntr_el0[24] = PMEVCNTR24_EL0.get();
            pmevtyper_el0[24] = PMEVTYPER24_EL0.get();
        }
        25 => {
            pmevcntr_el0[25] = PMEVCNTR25_EL0.get();
            pmevtyper_el0[25] = PMEVTYPER25_EL0.get();
        }
        26 => {
            pmevcntr_el0[26] = PMEVCNTR26_EL0.get();
            pmevtyper_el0[26] = PMEVTYPER26_EL0.get();
        }
        27 => {
            pmevcntr_el0[27] = PMEVCNTR27_EL0.get();
            pmevtyper_el0[27] = PMEVTYPER27_EL0.get();
        }
        28 => {
            pmevcntr_el0[28] = PMEVCNTR28_EL0.get();
            pmevtyper_el0[28] = PMEVTYPER28_EL0.get();
        }
        29 => {
            pmevcntr_el0[29] = PMEVCNTR29_EL0.get();
            pmevtyper_el0[29] = PMEVTYPER29_EL0.get();
        }
        30 => {
            pmevcntr_el0[30] = PMEVCNTR30_EL0.get();
            pmevtyper_el0[30] = PMEVTYPER30_EL0.get();
        }
        _ => warn!("Invalid PMEV index"),
    }
}

fn load_pmev(n: usize, cntr_val: u64, typer_val: u64) {
    match n {
        0 => {
            PMEVCNTR0_EL0.set(cntr_val);
            PMEVTYPER0_EL0.set(typer_val);
        }
        1 => {
            PMEVCNTR1_EL0.set(cntr_val);
            PMEVTYPER1_EL0.set(typer_val);
        }
        2 => {
            PMEVCNTR2_EL0.set(cntr_val);
            PMEVTYPER2_EL0.set(typer_val);
        }
        3 => {
            PMEVCNTR3_EL0.set(cntr_val);
            PMEVTYPER3_EL0.set(typer_val);
        }
        4 => {
            PMEVCNTR4_EL0.set(cntr_val);
            PMEVTYPER4_EL0.set(typer_val);
        }
        5 => {
            PMEVCNTR5_EL0.set(cntr_val);
            PMEVTYPER5_EL0.set(typer_val);
        }
        6 => {
            PMEVCNTR6_EL0.set(cntr_val);
            PMEVTYPER6_EL0.set(typer_val);
        }
        7 => {
            PMEVCNTR7_EL0.set(cntr_val);
            PMEVTYPER7_EL0.set(typer_val);
        }
        8 => {
            PMEVCNTR8_EL0.set(cntr_val);
            PMEVTYPER8_EL0.set(typer_val);
        }
        9 => {
            PMEVCNTR9_EL0.set(cntr_val);
            PMEVTYPER9_EL0.set(typer_val);
        }
        10 => {
            PMEVCNTR10_EL0.set(cntr_val);
            PMEVTYPER10_EL0.set(typer_val);
        }
        11 => {
            PMEVCNTR11_EL0.set(cntr_val);
            PMEVTYPER11_EL0.set(typer_val);
        }
        12 => {
            PMEVCNTR12_EL0.set(cntr_val);
            PMEVTYPER12_EL0.set(typer_val);
        }
        13 => {
            PMEVCNTR13_EL0.set(cntr_val);
            PMEVTYPER13_EL0.set(typer_val);
        }
        14 => {
            PMEVCNTR14_EL0.set(cntr_val);
            PMEVTYPER14_EL0.set(typer_val);
        }
        15 => {
            PMEVCNTR15_EL0.set(cntr_val);
            PMEVTYPER15_EL0.set(typer_val);
        }
        16 => {
            PMEVCNTR16_EL0.set(cntr_val);
            PMEVTYPER16_EL0.set(typer_val);
        }
        17 => {
            PMEVCNTR17_EL0.set(cntr_val);
            PMEVTYPER17_EL0.set(typer_val);
        }
        18 => {
            PMEVCNTR18_EL0.set(cntr_val);
            PMEVTYPER18_EL0.set(typer_val);
        }
        19 => {
            PMEVCNTR19_EL0.set(cntr_val);
            PMEVTYPER19_EL0.set(typer_val);
        }
        20 => {
            PMEVCNTR20_EL0.set(cntr_val);
            PMEVTYPER20_EL0.set(typer_val);
        }
        21 => {
            PMEVCNTR21_EL0.set(cntr_val);
            PMEVTYPER21_EL0.set(typer_val);
        }
        22 => {
            PMEVCNTR22_EL0.set(cntr_val);
            PMEVTYPER22_EL0.set(typer_val);
        }
        23 => {
            PMEVCNTR23_EL0.set(cntr_val);
            PMEVTYPER23_EL0.set(typer_val);
        }
        24 => {
            PMEVCNTR24_EL0.set(cntr_val);
            PMEVTYPER24_EL0.set(typer_val);
        }
        25 => {
            PMEVCNTR25_EL0.set(cntr_val);
            PMEVTYPER25_EL0.set(typer_val);
        }
        26 => {
            PMEVCNTR26_EL0.set(cntr_val);
            PMEVTYPER26_EL0.set(typer_val);
        }
        27 => {
            PMEVCNTR27_EL0.set(cntr_val);
            PMEVTYPER27_EL0.set(typer_val);
        }
        28 => {
            PMEVCNTR28_EL0.set(cntr_val);
            PMEVTYPER28_EL0.set(typer_val);
        }
        29 => {
            PMEVCNTR29_EL0.set(cntr_val);
            PMEVTYPER29_EL0.set(typer_val);
        }
        30 => {
            PMEVCNTR30_EL0.set(cntr_val);
            PMEVTYPER30_EL0.set(typer_val);
        }
        _ => warn!("Invalid PMEV index"),
    }
}

pub fn set_pmev_regs(
    cnt: usize,
    pmevcntr_el0: &[u64; MAX_EVCNT],
    pmevtyper_el0: &[u64; MAX_EVCNT],
) {
    if cnt > MAX_EVCNT {
        error!("Index out of bounds");
        return;
    }
    for i in 0..cnt {
        load_pmev(i, pmevcntr_el0[i], pmevtyper_el0[i]);
    }
}

pub fn get_pmev_regs(
    cnt: usize,
    pmevcntr_el0: &mut [u64; MAX_EVCNT],
    pmevtyper_el0: &mut [u64; MAX_EVCNT],
) {
    if cnt > MAX_EVCNT {
        error!("Index out of bounds");
        return;
    }
    for i in 0..cnt {
        store_pmev(i, pmevcntr_el0, pmevtyper_el0);
    }
}
