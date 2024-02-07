use super::timer;
use crate::cpu::get_cpu_id;
use crate::gic;
use crate::realm::registry::get_realm;
use crate::realm::vcpu::VCPU;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;

use armv9a::regs::*;

#[repr(C)]
#[derive(Default, Debug)]
pub struct Context {
    pub gp_regs: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
    pub sys_regs: SystemRegister,
    pub gic_state: GICRegister,
    pub timer: TimerRegister,
    pub fp_regs: [u128; 32],
}

pub fn set_reg(id: usize, vcpu: usize, register: usize, value: usize) -> Result<(), Error> {
    match register {
        0..=30 => {
            get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                .lock()
                .context
                .gp_regs[register] = value as u64;
            Ok(())
        }
        31 => {
            get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                .lock()
                .context
                .elr = value as u64;
            Ok(())
        }
        32 => {
            get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                .lock()
                .context
                .spsr = value as u64;
            Ok(())
        }
        _ => Err(Error::RmiErrorInput),
    }?;
    Ok(())
}

pub fn get_reg(id: usize, vcpu: usize, register: usize) -> Result<usize, Error> {
    match register {
        0..=30 => {
            let value = get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                .lock()
                .context
                .gp_regs[register];
            Ok(value as usize)
        }
        31 => {
            let value = get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                .lock()
                .context
                .elr;
            Ok(value as usize)
        }
        _ => Err(Error::RmiErrorInput),
    }
}

impl crate::realm::vcpu::Context for Context {
    fn new() -> Self {
        // Set appropriate sys registers
        // TODO: enable floating point
        // CPTR_EL2, CPACR_EL1, update vectors.s, etc..
        Self {
            spsr: SPSR_EL2::D | SPSR_EL2::A | SPSR_EL2::I | SPSR_EL2::F | (SPSR_EL2::M & 0b0101),
            ..Default::default()
        }
    }

    unsafe fn into_current(vcpu: &mut VCPU<Self>) {
        vcpu.pcpu = Some(get_cpu_id());
        vcpu.context.sys_regs.vmpidr = vcpu.pcpu.unwrap() as u64;
        TPIDR_EL2.set(vcpu as *const _ as u64);
        gic::restore_state(vcpu);
        timer::restore_state(vcpu);
    }

    unsafe fn from_current(vcpu: &mut VCPU<Self>) {
        gic::save_state(vcpu);
        timer::save_state(vcpu);
        vcpu.pcpu = None;
        //vcpu.context.sys_regs.vmpidr = 0u64;
        //TPIDR_EL2.set(0u64);
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
