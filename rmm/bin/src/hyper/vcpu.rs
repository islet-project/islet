use super::context::Context;
use super::vm::{VM, VMS};
use crate::aarch64::cpu::get_cpu_id;
use crate::config::PRIMARY_VM_ID;
use alloc::sync::{Arc, Weak};
use spin::Mutex;

pub const VCPU_INIT: Option<Arc<Mutex<VCPU>>> = None;

#[repr(C)]
#[derive(Default, Debug)]
pub struct VCPU {
    pub context: Context,
    pub vm: Weak<Mutex<VM>>, // VM struct the VCPU belongs to
    pub state: State,
    pub pcpu: u32,
}

impl VCPU {
    pub unsafe fn vcpu_init() {
        let cpu = get_cpu_id();

        let primary_vm = VM::get_vm(PRIMARY_VM_ID).unwrap();

        let mut vcpu: VCPU = Default::default();
        vcpu.vm = Arc::downgrade(&primary_vm);
        vcpu.state = State::Init;
        vcpu.pcpu = cpu as u32;

        let mut primary_vm = primary_vm.lock();
        primary_vm.vcpus[cpu] = Some(Arc::new(Mutex::new(vcpu)));

        // TPIDR_EL2.set(&current_vcpu_context as *const u64 as u64);
    }
}

#[derive(Debug)]
pub enum State {
    Init,
    Ready,
    Running,
    Trapped,
    Destroy,
}

impl Default for State {
    fn default() -> Self {
        State::Init
    }
}
