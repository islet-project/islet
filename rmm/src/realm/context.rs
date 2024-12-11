use super::timer;
use crate::gic;
use crate::rec::Rec;
use crate::rmi::error::Error;
use crate::simd;
use crate::simd::SimdRegister;

use aarch64_cpu::registers::*;

#[repr(C)]
#[derive(Default, Debug)]
pub struct Context {
    pub gp_regs: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
    pub sys_regs: SystemRegister,
    pub gic_state: GICRegister,
    pub timer: TimerRegister,
    pub simd: SimdRegister,
}

pub struct RegOffset;
impl RegOffset {
    pub const PC: usize = 31;
    pub const PSTATE: usize = 32;
    pub const SCTLR: usize = 40;
}

pub fn set_reg(rec: &mut Rec<'_>, register: usize, value: usize) -> Result<(), Error> {
    match register {
        0..=30 => {
            rec.context.gp_regs[register] = value as u64;
            Ok(())
        }
        RegOffset::PC => {
            rec.context.elr = value as u64;
            Ok(())
        }
        RegOffset::PSTATE => {
            rec.context.spsr = value as u64;
            Ok(())
        }
        _ => Err(Error::RmiErrorInput),
    }?;
    Ok(())
}

pub fn get_reg(rec: &Rec<'_>, register: usize) -> Result<usize, Error> {
    match register {
        0..=30 => {
            let value = rec.context.gp_regs[register];
            Ok(value as usize)
        }
        RegOffset::PC => {
            let value = rec.context.elr;
            Ok(value as usize)
        }
        _ => Err(Error::RmiErrorInput),
    }
}

impl Context {
    pub fn new() -> Self {
        // Set appropriate sys registers
        // TODO: enable floating point
        // CPTR_EL2, CPACR_EL1, update vectors.s, etc..
        Self {
            spsr: (SPSR_EL2::D.mask << SPSR_EL2::D.shift)
                | (SPSR_EL2::A.mask << SPSR_EL2::A.shift)
                | (SPSR_EL2::I.mask << SPSR_EL2::I.shift)
                | (SPSR_EL2::F.mask << SPSR_EL2::F.shift)
                | (SPSR_EL2::M.mask & u64::from(SPSR_EL2::M::EL1h)) << SPSR_EL2::M.shift,
            ..Default::default()
        }
    }

    /// Restores the current execution context from the given `Rec`.
    ///
    /// # Safety
    ///
    /// - This function modifies processor-specific registers and state;
    ///   ensure that this is safe in the current execution context.
    pub unsafe fn into_current(rec: &Rec<'_>) {
        TPIDR_EL2.set(rec as *const _ as u64);
        gic::restore_state(rec);
        timer::restore_state(rec);
        #[cfg(not(any(test, miri)))]
        simd::restore_state(rec);
    }

    /// Saves the current execution context into the given `Rec` record.
    ///
    /// # Safety
    ///
    /// - This function reads and modifies processor-specific registers and state;
    ///   ensure that this is appropriate in the current execution context.
    pub unsafe fn from_current(rec: &mut Rec<'_>) {
        gic::save_state(rec);
        timer::save_state(rec);
        #[cfg(not(any(test, miri)))]
        simd::save_state(rec);
    }
}

/// Generic Interrupt Controller Registers
#[repr(C)]
#[derive(Default, Debug)]
pub struct GICRegister {
    // Interrupt Controller Hyp Active Priorities Group 0 Registers
    pub ich_ap0r_el2: [u64; 4],
    // Interrupt Controller Hyp Active Priorities Group 1 Registers
    pub ich_ap1r_el2: [u64; 4],
    // GICv3 Virtual Machine Control Register
    pub ich_vmcr_el2: u64,
    // Interrupt Controller Hyp Control Register
    pub ich_hcr_el2: u64,
    // GICv3 List Registers
    pub ich_lr_el2: [u64; 16],
    // GICv3 Maintenance Interrupt State Register
    pub ich_misr_el2: u64,
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct SystemRegister {
    pub sp: u64,
    pub sp_el0: u64,
    pub esr_el1: u64,
    pub vbar: u64,
    pub ttbr0: u64,
    pub ttbr1: u64,
    pub mair: u64,
    pub amair: u64,
    pub tcr: u64,
    pub tpidr: u64,
    pub tpidr_el0: u64,
    pub tpidrro: u64,
    pub actlr: u64,
    pub vmpidr: u64,
    pub csselr: u64,
    pub cpacr: u64,
    pub afsr0: u64,
    pub afsr1: u64,
    pub far: u64,
    pub contextidr: u64,
    pub cntkctl: u64,
    pub par: u64,
    pub vttbr: u64,
    pub esr_el2: u64,
    pub hpfar: u64,
    pub sctlr: u64,
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct TimerRegister {
    pub cntvoff_el2: u64,
    pub cntv_cval_el0: u64,
    pub cntv_ctl_el0: u64,
    pub cntpoff_el2: u64,
    pub cntp_cval_el0: u64,
    pub cntp_ctl_el0: u64,
    pub cnthctl_el2: u64,
}
