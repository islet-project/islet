use super::vcpu::{VCPU, VCPU_INIT};
use crate::config::MAX_VCPUS;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::mutex::Mutex;

const VM_INIT: Option<Arc<Mutex<VM>>> = None;
pub static mut VMS: Vec<Arc<Mutex<VM>>> = Vec::new();

#[derive(Default, Debug)]
pub struct VM {
    pub id: u32,
    pub state: State,
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
            state: State::Init,
            vcpus: [VCPU_INIT; MAX_VCPUS],
            num_vcpu: 0,
        }
    }

    pub unsafe fn initialize(&mut self, id: usize, num_vcpu: usize) {
        // TODO: initialize pagetable
        self.id = id as u32;
        self.num_vcpu = num_vcpu as u32;
        self.state = State::Init;
    }

    pub fn get_vm(id: usize) -> Option<Arc<Mutex<VM>>> {
        unsafe {
            match VMS.iter().find(|&vm| vm.lock().id == id as u32) {
                Some(ref mut arcvm) => Some(Arc::clone(arcvm)),
                _ => None,
            }
        }
    }
}

#[derive(Debug)]
pub enum State {
    Init,
    Ready,
    Running,
    Destroy,
}

impl Default for State {
    fn default() -> Self {
        State::Init
    }
}
