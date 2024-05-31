use crate::rec::Rec;
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;

use aarch64_cpu::registers::*;
use lazy_static::lazy_static;

// Interrupt Controller List Registers (ICH_LR)
const ICH_LR_PRIORITY_WIDTH: u64 = 8;

const ICH_HCR_EL2_NS_MASK: u64 = (ICH_HCR_EL2::UIE.mask << ICH_HCR_EL2::UIE.shift)
    | (ICH_HCR_EL2::LRENPIE.mask << ICH_HCR_EL2::LRENPIE.shift)
    | (ICH_HCR_EL2::NPIE.mask << ICH_HCR_EL2::NPIE.shift)
    | (ICH_HCR_EL2::VGrp1DIE.mask << ICH_HCR_EL2::VGrp1DIE.shift)
    | (ICH_HCR_EL2::VGrp1EIE.mask << ICH_HCR_EL2::VGrp1EIE.shift)
    | (ICH_HCR_EL2::VGrp0DIE.mask << ICH_HCR_EL2::VGrp0DIE.shift)
    | (ICH_HCR_EL2::VGrp0EIE.mask << ICH_HCR_EL2::VGrp0EIE.shift)
    | (ICH_HCR_EL2::TDIR.mask << ICH_HCR_EL2::TDIR.shift);

const ICH_HCR_EL2_EOI_COUNT_WIDTH: usize = 5;
const ICH_HCR_EL2_EOI_COUNT_MASK: u64 =
    ((!0u64) >> (64 - ICH_HCR_EL2_EOI_COUNT_WIDTH)) << ICH_HCR_EL2::EOIcount.shift;

const MAX_SPI_ID: u64 = 1019;

const MIN_EPPI_ID: u64 = 1056;
const MAX_EPPI_ID: u64 = 1119;

const MIN_ESPI_ID: u64 = 4096;
const MAX_ESPI_ID: u64 = 5119;

const MIN_LPI_ID: u64 = 8192;

#[allow(dead_code)]
pub struct GicFeatures {
    pub nr_lrs: usize,
    pub nr_aprs: usize,
    pub pri_res0_mask: u64,
    pub max_vintid: u64,
    pub ext_range: bool,
}

lazy_static! {
    pub static ref GIC_FEATURES: GicFeatures = {
        trace!("read gic features");
        let nr_lrs = ICH_VTR_EL2.read(ICH_VTR_EL2::ListRegs) as usize;
        trace!("nr_lrs (LIST) {}", nr_lrs);
        let id = ICH_VTR_EL2.read(ICH_VTR_EL2::IDbits);
        let max_vintid = if id == 0 {
            (1u64 << 16) - 1
        } else {
            (1u64 << 24) - 1
        };
        trace!("id {} max_vintid {}", id, max_vintid);
        let pre = ICH_VTR_EL2.read(ICH_VTR_EL2::PREbits) + 1;
        let nr_aprs = (1 << (pre - 5)) - 1;
        trace!("pre {}, nr_aprs {}", pre, nr_aprs);
        let pri = ICH_VTR_EL2.read(ICH_VTR_EL2::PRIbits) + 1;
        let pri_res0_mask = (1u64 << (ICH_LR_PRIORITY_WIDTH - pri)) - 1;
        trace!("pri {} pri_res0_mask {}", pri, pri_res0_mask);
        let ext_range = ICC_CTLR_EL1.read(ICC_CTLR_EL1::ExtRange) != 0;
        trace!("icc_ctlr ext_range {}", ext_range);
        GicFeatures {
            nr_lrs,
            nr_aprs,
            pri_res0_mask,
            max_vintid,
            ext_range,
        }
    };
}

pub fn init_gic(rec: &mut Rec<'_>) {
    let gic_state = &mut rec.context.gic_state;
    gic_state.ich_hcr_el2 = (ICH_HCR_EL2::En.mask << ICH_HCR_EL2::En.shift)
        + (ICH_HCR_EL2::vSGIEOICount.mask << ICH_HCR_EL2::vSGIEOICount.shift)
        + (ICH_HCR_EL2::DVIM.mask << ICH_HCR_EL2::DVIM.shift)
}

fn set_lr(i: usize, val: u64) {
    match i {
        0 => ICH_LR0_EL2.set(val),
        1 => ICH_LR1_EL2.set(val),
        2 => ICH_LR2_EL2.set(val),
        3 => ICH_LR3_EL2.set(val),
        4 => ICH_LR4_EL2.set(val),
        5 => ICH_LR5_EL2.set(val),
        6 => ICH_LR6_EL2.set(val),
        7 => ICH_LR7_EL2.set(val),
        8 => ICH_LR8_EL2.set(val),
        9 => ICH_LR9_EL2.set(val),
        10 => ICH_LR10_EL2.set(val),
        11 => ICH_LR11_EL2.set(val),
        12 => ICH_LR12_EL2.set(val),
        13 => ICH_LR13_EL2.set(val),
        14 => ICH_LR14_EL2.set(val),
        15 => ICH_LR15_EL2.set(val),
        _ => {}
    }
}

fn set_ap0r(i: usize, val: u64) {
    match i {
        0 => ICH_AP0R0_EL2.set(val),
        1 => ICH_AP0R1_EL2.set(val),
        2 => ICH_AP0R2_EL2.set(val),
        3 => ICH_AP0R3_EL2.set(val),
        _ => {}
    }
}

fn set_ap1r(i: usize, val: u64) {
    match i {
        0 => ICH_AP1R0_EL2.set(val),
        1 => ICH_AP1R1_EL2.set(val),
        2 => ICH_AP1R2_EL2.set(val),
        3 => ICH_AP1R3_EL2.set(val),
        _ => {}
    }
}

fn get_lr(i: usize) -> u64 {
    match i {
        0 => ICH_LR0_EL2.get(),
        1 => ICH_LR1_EL2.get(),
        2 => ICH_LR2_EL2.get(),
        3 => ICH_LR3_EL2.get(),
        4 => ICH_LR4_EL2.get(),
        5 => ICH_LR5_EL2.get(),
        6 => ICH_LR6_EL2.get(),
        7 => ICH_LR7_EL2.get(),
        8 => ICH_LR8_EL2.get(),
        9 => ICH_LR9_EL2.get(),
        10 => ICH_LR10_EL2.get(),
        11 => ICH_LR11_EL2.get(),
        12 => ICH_LR12_EL2.get(),
        13 => ICH_LR13_EL2.get(),
        14 => ICH_LR14_EL2.get(),
        15 => ICH_LR15_EL2.get(),
        _ => unreachable!(),
    }
}

fn get_ap0r(i: usize) -> u64 {
    match i {
        0 => ICH_AP0R0_EL2.get(),
        1 => ICH_AP0R1_EL2.get(),
        2 => ICH_AP0R2_EL2.get(),
        3 => ICH_AP0R3_EL2.get(),
        _ => unreachable!(),
    }
}

fn get_ap1r(i: usize) -> u64 {
    match i {
        0 => ICH_AP1R0_EL2.get(),
        1 => ICH_AP1R1_EL2.get(),
        2 => ICH_AP1R2_EL2.get(),
        3 => ICH_AP1R3_EL2.get(),
        _ => unreachable!(),
    }
}

pub fn restore_state(rec: &Rec<'_>) {
    let gic_state = &rec.context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;
    let nr_aprs = GIC_FEATURES.nr_aprs;

    for i in 0..=nr_lrs {
        set_lr(i, gic_state.ich_lr_el2[i]);
    }
    for i in 0..=nr_aprs {
        set_ap0r(i, gic_state.ich_ap0r_el2[i]);
        set_ap1r(i, gic_state.ich_ap1r_el2[i]);
    }
    ICH_VMCR_EL2.set(gic_state.ich_vmcr_el2);
    ICH_HCR_EL2.set(gic_state.ich_hcr_el2);
}

pub fn save_state(rec: &mut Rec<'_>) {
    let gic_state = &mut rec.context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;
    let nr_aprs = GIC_FEATURES.nr_aprs;

    for i in 0..=nr_lrs {
        gic_state.ich_lr_el2[i] = get_lr(i);
    }
    for i in 0..=nr_aprs {
        gic_state.ich_ap0r_el2[i] = get_ap0r(i);
        gic_state.ich_ap1r_el2[i] = get_ap1r(i);
    }

    gic_state.ich_vmcr_el2 = ICH_VMCR_EL2.get();
    gic_state.ich_hcr_el2 = ICH_HCR_EL2.get();
    gic_state.ich_misr_el2 = ICH_MISR_EL2.get();

    ICH_HCR_EL2.set(gic_state.ich_hcr_el2 & !(ICH_HCR_EL2::En.mask << ICH_HCR_EL2::En.shift));
}

pub fn receive_state_from_host(rec: &mut Rec<'_>, run: &Run) -> Result<(), Error> {
    let gic_state = &mut rec.context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;

    gic_state.ich_lr_el2[..nr_lrs].copy_from_slice(&run.entry_gic_lrs()[..nr_lrs]);
    gic_state.ich_hcr_el2 &= !ICH_HCR_EL2_NS_MASK;
    gic_state.ich_hcr_el2 |= run.entry_gic_hcr() & ICH_HCR_EL2_NS_MASK;
    Ok(())
}

pub fn send_state_to_host(rec: &Rec<'_>, run: &mut Run) -> Result<(), Error> {
    let gic_state = &rec.context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;

    run.exit_gic_lrs_mut()[..nr_lrs].copy_from_slice(&gic_state.ich_lr_el2[..nr_lrs]);
    run.set_gic_misr(gic_state.ich_misr_el2);
    run.set_gic_vmcr(gic_state.ich_vmcr_el2);
    run.set_gic_hcr(gic_state.ich_hcr_el2 & (ICH_HCR_EL2_EOI_COUNT_MASK | ICH_HCR_EL2_NS_MASK));
    Ok(())
}

fn valid_vintid(intid: u64) -> bool {
    /* Check for INTID [0..1019] and [8192..] */
    if intid <= MAX_SPI_ID || (intid >= MIN_LPI_ID && intid <= GIC_FEATURES.max_vintid) {
        return true;
    }

    /*
     * If extended INTID range sopported, check for
     * Extended PPI [1056..1119] and Extended SPI [4096..5119]
     */
    if GIC_FEATURES.ext_range {
        return (intid >= MIN_EPPI_ID && intid <= MAX_EPPI_ID)
            || (intid >= MIN_ESPI_ID && intid <= MAX_ESPI_ID);
    }

    false
}

pub fn validate_state(run: &Run) -> bool {
    let hcr = run.entry_gic_hcr();

    /* Validate rec_entry.gicv3_hcr MBZ bits */
    if (hcr & !ICH_HCR_EL2_NS_MASK) != 0 {
        return false;
    }

    for i in 0..GIC_FEATURES.nr_lrs {
        let lrs = run.entry_gic_lrs();
        let lr = lrs[i];

        let state = (lr >> ICH_LR0_EL2::State.shift) & ICH_LR0_EL2::State.mask;
        let vintid = (lr >> ICH_LR0_EL2::vINTID.shift) & ICH_LR0_EL2::vINTID.mask;
        let priority = (lr >> ICH_LR0_EL2::Priority.shift) & ICH_LR0_EL2::Priority.mask;

        let pintid_mask = ICH_LR0_EL2::pINTID.mask << ICH_LR0_EL2::pINTID.shift;
        let eoi_mask = ICH_LR0_EL2::EOI.mask << ICH_LR0_EL2::EOI.shift;
        let only_eoi = pintid_mask & !eoi_mask;

        let hw = (lr >> ICH_LR0_EL2::HW.shift) & ICH_LR0_EL2::HW.mask;

        if state == ICH_LR0_EL2::State::Invalid.into() {
            continue;
        }

        /* The RMM Specification imposes the constraint that HW == '0' */
        if hw != 0
            /* Check RES0 bits in the Priority field */
            || priority & GIC_FEATURES.pri_res0_mask != 0
            /* Only the EOI bit in the pINTID is allowed to be set */
            || lr & only_eoi != 0
            /* Check if vINTID is in the valid range */
            || !valid_vintid(vintid)
        {
            return false;
        }

        /*
         * Behavior is UNPREDICTABLE if two or more List Registers
         * specify the same vINTID.
         */
        for j in i + 1..=GIC_FEATURES.nr_lrs {
            let lrs = run.entry_gic_lrs();
            let lr = lrs[j];

            let vintid_2 = (lr >> ICH_LR0_EL2::vINTID.shift) & ICH_LR0_EL2::vINTID.mask;
            let state = (lr >> ICH_LR0_EL2::State.shift) & ICH_LR0_EL2::State.mask;

            if state == ICH_LR0_EL2::State::Invalid.into() {
                continue;
            }

            if vintid == vintid_2 {
                return false;
            }
        }
    }

    true
}
