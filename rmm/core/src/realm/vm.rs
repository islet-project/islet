use super::vcpu::{Context, VCPU};

use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::mutex::Mutex;

extern crate alloc;

#[derive(Debug)]
pub struct VM<T: Context> {
    id: usize,
    pub state: State,
    pub vcpus: Vec<Arc<Mutex<VCPU<T>>>>,
    // TODO: add pagetable
}

impl<T: Context> VM<T> {
    pub const fn new(id: usize) -> Self {
        // TODO: initialize pagetable
        Self {
            id: id,
            state: State::Init,
            vcpus: Vec::new(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug)]
pub enum State {
    Init,
    Ready,
    Running,
    Destroy,
}
