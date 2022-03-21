use super::vcpu::{Context, VCPU};
use super::vmem::IPATranslation;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::mutex::Mutex;

extern crate alloc;

#[derive(Debug)]
pub struct VM<T: Context> {
    id: usize,
    pub state: State,
    pub vcpus: Vec<Arc<Mutex<VCPU<T>>>>,
    pub page_table: Arc<Mutex<Box<dyn IPATranslation>>>,
    // TODO: add pagetable
}

impl<T: Context> VM<T> {
    pub const fn new(id: usize, page_table: Arc<Mutex<Box<dyn IPATranslation>>>) -> Self {
        Self {
            id: id,
            state: State::Init,
            vcpus: Vec::new(),
            page_table: page_table,
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
