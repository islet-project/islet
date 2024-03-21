use crate::realm::rd::Rd;
use crate::realm::vcpu::VCPU;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::rec::run::Run;

use armv9a::regs::*;
use lazy_static::lazy_static;

// Interrupt Controller List Registers (ICH_LR)
const ICH_LR_PRIORITY_WIDTH: u64 = 8;

// Interrupt Controller Hyp Control Register (ICH_HCR)
// Global enable bit for the virtual CPU interface.
const ICH_HCR_EL2_EN_BIT: u64 = 1 << 0;
// Underflow Interrupt Enable
const ICH_HCR_EL2_UIE_BIT: u64 = 1 << 1;
// List Register Entry Not Present Interrupt Enable
const ICH_HCR_EL2_LRENPIE_BIT: u64 = 1 << 2;
// No Pending Interrupt Enable
const ICH_HCR_EL2_NPIE_BIT: u64 = 1 << 3;
// VM Group 0 Enabled Interrupt Enable
const ICH_HCR_EL2_VGRP0EIE_BIT: u64 = 1 << 4;
// VM Group 0 Disabled Interrupt Enable
const ICH_HCR_EL2_VGRP0DIE_BIT: u64 = 1 << 5;
// VM Group 1 Enabled Interrupt Enable
const ICH_HCR_EL2_VGRP1EIE_BIT: u64 = 1 << 6;
// VM Group 1 Disabled Interrupt Enable
const ICH_HCR_EL2_VGRP1DIE_BIT: u64 = 1 << 7;
// Deactivation of virtual SGIs can increment ICH_HCR_EL2.EOIcount
const ICH_HCR_EL2_VSGIEEOICOUNT_BIT: u64 = 1 << 8;
// When FEAT_GICv3_TDIR is implemented, Trap EL1 writes to ICC_DIR_EL1 and ICV_DIR_EL1.
const ICH_HCR_EL2_TDIR_BIT: u64 = 1 << 14;
// ICH_HCR_EL2_DVIM_BIT
const ICH_HCR_EL2_DVIM_BIT: u64 = 1 << 15;

pub const ICH_HCR_EL2_NS_MASK: u64 = ICH_HCR_EL2_UIE_BIT
    | ICH_HCR_EL2_LRENPIE_BIT
    | ICH_HCR_EL2_NPIE_BIT
    | ICH_HCR_EL2_VGRP0EIE_BIT
    | ICH_HCR_EL2_VGRP0DIE_BIT
    | ICH_HCR_EL2_VGRP1EIE_BIT
    | ICH_HCR_EL2_VGRP1DIE_BIT
    | ICH_HCR_EL2_TDIR_BIT;

const ICH_HCR_EL2_EOI_COUNT_SHIFT: usize = 27;
const ICH_HCR_EL2_EOI_COUNT_WIDTH: usize = 5;
pub const ICH_HCR_EL2_EOI_COUNT_MASK: u64 =
    ((!0u64) >> (64 - ICH_HCR_EL2_EOI_COUNT_WIDTH)) << ICH_HCR_EL2_EOI_COUNT_SHIFT;

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
        let nr_lrs = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::LIST) } as usize;
        trace!("nr_lrs (LIST) {}", nr_lrs);
        let id = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::ID) };
        let max_vintid = if id == 0 {
            (1u64 << 16) - 1
        } else {
            (1u64 << 24) - 1
        };
        trace!("id {} max_vintid {}", id, max_vintid);
        let pre = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::PRE) } + 1;
        let nr_aprs = (1 << (pre - 5)) - 1;
        trace!("pre {}, nr_aprs {}", pre, nr_aprs);
        let pri = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::PRI) } + 1;
        let pri_res0_mask = (1u64 << (ICH_LR_PRIORITY_WIDTH - pri)) - 1;
        trace!("pri {} pri_res0_mask {}", pri, pri_res0_mask);
        let ext_range = unsafe { ICC_CTLR_EL1.get_masked_value(ICC_CTLR_EL1::EXT_RANGE) != 0 };
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

pub fn init_gic(vcpu: &mut VCPU) {
    let gic_state = &mut vcpu.context.gic_state;
    gic_state.ich_hcr_el2 =
        ICH_HCR_EL2_EN_BIT | ICH_HCR_EL2_VSGIEEOICOUNT_BIT | ICH_HCR_EL2_DVIM_BIT
}

fn set_lr(i: usize, val: u64) {
    match i {
        0 => unsafe { ICH_LR0_EL2.set(val) },
        1 => unsafe { ICH_LR1_EL2.set(val) },
        2 => unsafe { ICH_LR2_EL2.set(val) },
        3 => unsafe { ICH_LR3_EL2.set(val) },
        4 => unsafe { ICH_LR4_EL2.set(val) },
        5 => unsafe { ICH_LR5_EL2.set(val) },
        6 => unsafe { ICH_LR6_EL2.set(val) },
        7 => unsafe { ICH_LR7_EL2.set(val) },
        8 => unsafe { ICH_LR8_EL2.set(val) },
        9 => unsafe { ICH_LR9_EL2.set(val) },
        10 => unsafe { ICH_LR10_EL2.set(val) },
        11 => unsafe { ICH_LR11_EL2.set(val) },
        12 => unsafe { ICH_LR12_EL2.set(val) },
        13 => unsafe { ICH_LR13_EL2.set(val) },
        14 => unsafe { ICH_LR14_EL2.set(val) },
        15 => unsafe { ICH_LR15_EL2.set(val) },
        _ => {}
    }
}

fn set_ap0r(i: usize, val: u64) {
    match i {
        0 => unsafe {
            ICH_AP0R0_EL2.set(val);
        },
        1 => unsafe {
            ICH_AP0R1_EL2.set(val);
        },
        2 => unsafe {
            ICH_AP0R2_EL2.set(val);
        },
        3 => unsafe {
            ICH_AP0R3_EL2.set(val);
        },
        _ => {}
    }
}

fn set_ap1r(i: usize, val: u64) {
    match i {
        0 => unsafe {
            ICH_AP1R0_EL2.set(val);
        },
        1 => unsafe {
            ICH_AP1R1_EL2.set(val);
        },
        2 => unsafe {
            ICH_AP1R2_EL2.set(val);
        },
        3 => unsafe {
            ICH_AP1R3_EL2.set(val);
        },
        _ => {}
    }
}

fn get_lr(i: usize) -> u64 {
    match i {
        0 => unsafe { ICH_LR0_EL2.get() },
        1 => unsafe { ICH_LR1_EL2.get() },
        2 => unsafe { ICH_LR2_EL2.get() },
        3 => unsafe { ICH_LR3_EL2.get() },
        4 => unsafe { ICH_LR4_EL2.get() },
        5 => unsafe { ICH_LR5_EL2.get() },
        6 => unsafe { ICH_LR6_EL2.get() },
        7 => unsafe { ICH_LR7_EL2.get() },
        8 => unsafe { ICH_LR8_EL2.get() },
        9 => unsafe { ICH_LR9_EL2.get() },
        10 => unsafe { ICH_LR10_EL2.get() },
        11 => unsafe { ICH_LR11_EL2.get() },
        12 => unsafe { ICH_LR12_EL2.get() },
        13 => unsafe { ICH_LR13_EL2.get() },
        14 => unsafe { ICH_LR14_EL2.get() },
        15 => unsafe { ICH_LR15_EL2.get() },
        _ => {
            unreachable!();
        }
    }
}

fn get_ap0r(i: usize) -> u64 {
    match i {
        0 => unsafe { ICH_AP0R0_EL2.get() },
        1 => unsafe { ICH_AP0R1_EL2.get() },
        2 => unsafe { ICH_AP0R2_EL2.get() },
        3 => unsafe { ICH_AP0R3_EL2.get() },
        _ => {
            unreachable!();
        }
    }
}

fn get_ap1r(i: usize) -> u64 {
    match i {
        0 => unsafe { ICH_AP1R0_EL2.get() },
        1 => unsafe { ICH_AP1R1_EL2.get() },
        2 => unsafe { ICH_AP1R2_EL2.get() },
        3 => unsafe { ICH_AP1R3_EL2.get() },
        _ => {
            unreachable!();
        }
    }
}

pub fn restore_state(vcpu: &VCPU) {
    let gic_state = &vcpu.context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;
    let nr_aprs = GIC_FEATURES.nr_aprs;

    for i in 0..=nr_lrs {
        set_lr(i, gic_state.ich_lr_el2[i]);
    }
    for i in 0..=nr_aprs {
        set_ap0r(i, gic_state.ich_ap0r_el2[i]);
        set_ap1r(i, gic_state.ich_ap1r_el2[i]);
    }
    unsafe { ICH_VMCR_EL2.set(gic_state.ich_vmcr_el2) };
    unsafe { ICH_HCR_EL2.set(gic_state.ich_hcr_el2) };
}

pub fn save_state(vcpu: &mut VCPU) {
    let gic_state = &mut vcpu.context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;
    let nr_aprs = GIC_FEATURES.nr_aprs;

    for i in 0..=nr_lrs {
        gic_state.ich_lr_el2[i] = get_lr(i);
    }
    for i in 0..=nr_aprs {
        gic_state.ich_ap0r_el2[i] = get_ap0r(i);
        gic_state.ich_ap1r_el2[i] = get_ap1r(i);
    }

    gic_state.ich_vmcr_el2 = unsafe { ICH_VMCR_EL2.get() };
    gic_state.ich_hcr_el2 = unsafe { ICH_HCR_EL2.get() };
    gic_state.ich_misr_el2 = unsafe { ICH_MISR_EL2.get() };

    unsafe { ICH_HCR_EL2.set(gic_state.ich_hcr_el2 & !ICH_HCR_EL2_EN_BIT) };
}

pub fn receive_state_from_host(rd: &Rd, vcpu: usize, run: &Run) -> Result<(), Error> {
    let vcpu = rd
        .vcpus
        .get(vcpu)
        .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
    let gic_state = &mut vcpu.lock().context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;

    gic_state.ich_lr_el2[..nr_lrs].copy_from_slice(&run.entry_gic_lrs()[..nr_lrs]);
    gic_state.ich_hcr_el2 &= !ICH_HCR_EL2_NS_MASK;
    gic_state.ich_hcr_el2 |= run.entry_gic_hcr() & ICH_HCR_EL2_NS_MASK;
    Ok(())
}

pub fn send_state_to_host(rd: &mut Rd, vcpu: usize, run: &mut Run) -> Result<(), Error> {
    let vcpu = rd
        .vcpus
        .get_mut(vcpu)
        .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
    let gic_state = &mut vcpu.lock().context.gic_state;
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
        let lr = ICH_LR::new(lrs[i]);
        let vintid = lr.get_masked_value(ICH_LR::VINTID);

        if lr.get_masked_value(ICH_LR::STATE) == ich_lr_state::INVALID {
            continue;
        }

        /* The RMM Specification imposes the constraint that HW == '0' */
        let pri = lr.get_masked_value(ICH_LR::PRIORITY);
        let pri_res0_mask = (1u64 << (ICH_LR_PRIORITY_WIDTH - pri)) - 1;
        if lr.get_masked(ICH_LR::HW) != 0
            /* Check RES0 bits in the Priority field */
            || pri_res0_mask != 0
            /* Only the EOI bit in the pINTID is allowed to be set */
            || lr.get_masked(ICH_LR::PINTID & !ICH_LR::EOI) != 0
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
            let lr = ICH_LR::new(lrs[j]);

            let vintid_2 = lr.get_masked_value(ICH_LR::VINTID);

            if lr.get_masked_value(ICH_LR::STATE) == ich_lr_state::INVALID {
                continue;
            }

            if vintid == vintid_2 {
                return false;
            }
        }
    }

    true
}
