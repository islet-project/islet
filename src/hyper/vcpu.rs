use super::vm::{VM, VMS};
use crate::aarch64::cpu::get_cpu_id;
use crate::config::PRIMARY_VM_ID;
use alloc::sync::Arc;
use spin::Mutex;

pub const VCPU_INIT: Option<Arc<Mutex<VCPU>>> = None;

#[repr(C)]
#[derive(Default, Debug)]
pub struct VCPU {
    pub context: VCPUContext,
    pub vm: Arc<Mutex<VM>>, // VM struct the VCPU belongs to
    pub state: VCPUState,
    pub pcpu: u32,
}

impl VCPU {
    pub unsafe fn vcpu_init() {
        let cpu = get_cpu_id();

        let primary_vm = VM::get_vm_as_mut_ref(PRIMARY_VM_ID).unwrap();
        let mut primary_vm = primary_vm.lock();

        let mut vcpu: VCPU = Default::default();
        vcpu.vm = match VMS[PRIMARY_VM_ID] {
            Some(ref arcvm) => Arc::clone(arcvm),
            None => panic!(),
        };
        vcpu.state = VCPUState::VCPUInit;
        vcpu.pcpu = cpu as u32;

        primary_vm.vcpus[cpu] = Some(Arc::new(Mutex::new(vcpu)));

        // TPIDR_EL2.set(&current_vcpu_context as *const u64 as u64);
    }
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct VCPUContext {
    pub gp_regs: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
    pub sys_regs: SystemRegister,
    // pub fp_regs: [u128; 32],
}

#[derive(Debug)]
pub enum VCPUState {
    VCPUInit,
    VCPUReady,
    VCPURunning,
    VCPUTrapped,
    VCPUDestroy,
}

impl Default for VCPUState {
    fn default() -> Self {
        VCPUState::VCPUInit
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
