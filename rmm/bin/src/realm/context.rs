use crate::aarch64::cpu::get_cpu_id;
use crate::aarch64::{HCR_EL2, SPSR_EL2, TPIDR_EL2};
use crate::config::{STACK_ALIGN, VM_STACK_SIZE};
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
    fn new() -> Self {
        let mut context: Self = Default::default();
        // TODO[1]: Set PC (and arg) for vCPU entry
        // context.elr = crate::dummy_main as u64;
        context.elr = 0x8806c000 as u64;

        // TODO[2]: Set appropriate sys registers (hcr, spsr, ..)
        context.sys_regs.sp = unsafe {
            alloc::alloc::alloc_zeroed(
                alloc::alloc::Layout::from_size_align(VM_STACK_SIZE, STACK_ALIGN).unwrap(),
            )
        } as u64;
        context.sys_regs.sp += VM_STACK_SIZE as u64;
        context.spsr =
            SPSR_EL2::D | SPSR_EL2::A | SPSR_EL2::I | SPSR_EL2::F | (SPSR_EL2::M & 0b0101);
        context.sys_regs.hcr = HCR_EL2::RW | HCR_EL2::TSC;

        // TODO[3]: enable floating point
        // CPTR_EL2, CPACR_EL1, update vectors.s, etc..

        context
    }

    unsafe fn set_current(vcpu: &mut VCPU<Self>) {
        vcpu.pcpu = Some(get_cpu_id());
        vcpu.context.sys_regs.vmpidr = vcpu.pcpu.unwrap() as u64;
        vcpu.state = rmm_core::realm::vcpu::State::Running;
        TPIDR_EL2.set(vcpu as *const _ as u64);
    }
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct SystemRegister {
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
    pub vmpidr: u64,
    pub csselr: u64,
    pub cpacr: u64,
    pub afsr0: u64,
    pub afsr1: u64,
    pub far: u64,
    pub contextidr: u64,
    pub cntkctl: u64,
    pub par: u64,
    pub hcr: u64,
    pub esr_el2: u64,
    pub hpfar: u64,
}
