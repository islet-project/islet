use super::vcpu::{VCPU, VCPU_INIT};
use crate::config::{MAX_VCPUS, MAX_VMS};
use alloc::sync::Arc;
use spin::mutex::Mutex;

const VM_INIT: Option<Arc<Mutex<VM>>> = None;
pub static mut VMS: [Option<Arc<Mutex<VM>>>; MAX_VMS] = [VM_INIT; MAX_VMS];

#[derive(Default, Debug)]
pub struct VM {
    pub id: u32,
    pub state: VMState,
    pub vcpus: [Option<Arc<Mutex<VCPU>>>; MAX_VCPUS],
    pub num_vcpu: u32,
    // TODO: add pagetable
}

impl VM {
    /// Returns an uninitialized `VM`.
    ///
    /// The VM must be initialized by calling `initialize()`
    /// before being started. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        Self {
            id: 0,
            state: VMState::VMInit,
            vcpus: [VCPU_INIT; MAX_VCPUS],
            num_vcpu: 0,
        }
    }

    pub unsafe fn initialize(&mut self, id: usize, num_vcpu: usize) {
        // TODO: assert VMS[id] is self
        // TODO: initialize pagetable
        self.id = id as u32;
        self.num_vcpu = num_vcpu as u32;
        self.state = VMState::VMInit;
    }

    pub fn get_vm_as_mut_ref(id: usize) -> Option<Arc<Mutex<VM>>> {
        unsafe {
            match VMS[id] {
                Some(ref mut arcvm) => Some(Arc::clone(arcvm)),
                _ => None,
            }
        }
    }
}

#[derive(Debug)]
pub enum VMState {
    VMInit,
    VMReady,
    VMRunning,
    VMDestroy,
}

impl Default for VMState {
    fn default() -> Self {
        VMState::VMInit
    }
}
