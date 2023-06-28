use crate::helper::ICH_VTR_EL2;
use crate::helper::{
    ICH_AP0R0_EL2, ICH_AP0R1_EL2, ICH_AP0R2_EL2, ICH_AP0R3_EL2, ICH_AP1R0_EL2, ICH_AP1R1_EL2,
    ICH_AP1R2_EL2, ICH_AP1R3_EL2,
};
use crate::helper::{
    ICH_HCR_EL2, ICH_LR0_EL2, ICH_LR10_EL2, ICH_LR11_EL2, ICH_LR12_EL2, ICH_LR13_EL2, ICH_LR14_EL2,
    ICH_LR15_EL2, ICH_LR1_EL2, ICH_LR2_EL2, ICH_LR3_EL2, ICH_LR4_EL2, ICH_LR5_EL2, ICH_LR6_EL2,
    ICH_LR7_EL2, ICH_LR8_EL2, ICH_LR9_EL2, ICH_MISR_EL2, ICH_VMCR_EL2,
};
use crate::realm::context::Context;
use lazy_static::lazy_static;
use monitor::realm::vcpu::VCPU;

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
// When FEAT_GICv3_TDIR is implemented, Trap EL1 writes to ICC_DIR_EL1 and ICV_DIR_EL1.
const ICH_HCR_EL2_TDIR_BIT: u64 = 1 << 14;

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

#[allow(dead_code)]
pub struct GicFeatures {
    pub nr_lrs: usize,
    pub nr_aprs: usize,
    pub pri_res0_mask: u64,
    pub max_vintid: u64,
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
        GicFeatures {
            nr_lrs,
            nr_aprs,
            pri_res0_mask,
            max_vintid,
        }
    };
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

pub fn restore_state(vcpu: &VCPU<Context>) {
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

pub fn save_state(vcpu: &mut VCPU<Context>) {
    let gic_state = &mut vcpu.context.gic_state;
    let nr_lrs = GIC_FEATURES.nr_lrs;
    let nr_aprs = GIC_FEATURES.nr_aprs;

    for i in 0..=nr_lrs {
        *&mut gic_state.ich_lr_el2[i] = get_lr(i);
    }
    for i in 0..=nr_aprs {
        *&mut gic_state.ich_ap0r_el2[i] = get_ap0r(i);
        *&mut gic_state.ich_ap1r_el2[i] = get_ap1r(i);
    }

    *&mut gic_state.ich_vmcr_el2 = unsafe { ICH_VMCR_EL2.get() };
    *&mut gic_state.ich_hcr_el2 = unsafe { ICH_HCR_EL2.get() };
    *&mut gic_state.ich_misr_el2 = unsafe { ICH_MISR_EL2.get() };

    unsafe { ICH_HCR_EL2.set(gic_state.ich_hcr_el2 & !ICH_HCR_EL2_EN_BIT) };
}
