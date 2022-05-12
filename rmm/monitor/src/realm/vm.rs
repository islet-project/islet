use super::mm::IPATranslation;
use super::vcpu::{Context, VCPU};

use crate::config::MAX_VCPUS;
use crate::error::{Error, ErrorKind};
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::mutex::Mutex;

extern crate alloc;

#[derive(Debug)]
pub struct VM<T: Context> {
    id: usize,
    me: Weak<Mutex<Self>>,
    pub state: State,
    pub vcpus: Vec<Arc<Mutex<VCPU<T>>>>,
    pub page_table: Arc<Mutex<Box<dyn IPATranslation>>>,
    // TODO: add pagetable
}

impl<T: Context + Default> VM<T> {
    pub fn new(id: usize, page_table: Arc<Mutex<Box<dyn IPATranslation>>>) -> Arc<Mutex<Self>> {
        Arc::<Mutex<Self>>::new_cyclic(|me| {
            let vcpus = Vec::with_capacity(MAX_VCPUS);

            let vm = Mutex::new(Self {
                id: id,
                me: me.clone(),
                state: State::Init,
                vcpus: vcpus,
                page_table: page_table,
            });

            vm
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn switch_to(&self, vcpu: usize) -> Result<(), Error> {
        self.vcpus
            .get(vcpu)
            .map(|vcpu| VCPU::into_current(&mut *vcpu.lock()))
            .ok_or(Error::new(ErrorKind::NotConnected))?;

        Ok(())
    }

    pub fn create_vcpu(&mut self, vcpu: usize) -> Result<(), Error> {
        if vcpu < MAX_VCPUS {
            self.vcpus.insert(vcpu, VCPU::new(self.me.clone()));
            self.vcpus
                .get(vcpu)
                .ok_or(Error::new(ErrorKind::NotConnected))?;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotConnected))
        }
    }
}

impl<T: Context> Drop for VM<T> {
    fn drop(&mut self) {
        info!("VM #{} was destroyed!", self.id);
    }
}

#[derive(Debug)]
pub enum State {
    Init,
    Ready,
    Running,
    Destroy,
}
