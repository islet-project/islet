use crate::aarch64::cpu::get_cpu_id;
use crate::aarch64::TPIDR_EL2;
use rmm_core::realm::vcpu::VCPU;

#[repr(C)]
#[derive(Default, Debug)]
pub struct Context {
    pub gp_regs: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
    pub sys_regs: SystemRegister,
    pub fp_regs: [u128; 32],
}

impl rmm_core::realm::vcpu::Context for Context {
    unsafe fn set_current(vcpu: &mut VCPU<Self>) {
        vcpu.pcpu = Some(get_cpu_id());
        TPIDR_EL2.set(vcpu as *const _ as u64);
    }
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct SystemRegister {
    pub spsr: u64,
    pub elr: u64,
    pub sctlr: u64,
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
    pub mpidr: u64,
    pub csselr: u64,
    pub cpacr: u64,
    pub afsr0: u64,
    pub afsr1: u64,
    pub far: u64,
    pub contextidr: u64,
    pub cntkctl: u64,
    pub par: u64,
    pub disr: u64,
    pub hcr: u64,
    pub esr_el2: u64,
    pub hpfar: u64,
}
