use super::vcpu::{Context, VCPU};
use super::vmem::IPATranslation;

use crate::error::{Error, ErrorKind};
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

impl<T: Context + Default> VM<T> {
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

    pub fn switch_to(&self, vcpu: usize) -> Result<(), Error> {
        self.vcpus
            .get(vcpu)
            .map(|vcpu| vcpu.lock().set_current())
            .ok_or(Error::new(ErrorKind::NotConnected))?;
        self.page_table.lock().set_mmu();

        Ok(())
    }
}

impl<T: Context> Drop for VM<T> {
    fn drop(&mut self) {
        //TODO unset pagetable
    }
}

#[derive(Debug)]
pub enum State {
    Init,
    Ready,
    Running,
    Destroy,
}
