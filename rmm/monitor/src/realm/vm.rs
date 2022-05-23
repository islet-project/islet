use super::mm::IPATranslation;
use super::vcpu::{Context, VCPU};

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
}

impl<T: Context + Default> VM<T> {
    pub fn new(id: usize, page_table: Arc<Mutex<Box<dyn IPATranslation>>>) -> Arc<Mutex<Self>> {
        Arc::<Mutex<Self>>::new_cyclic(|me| {
            let vcpus = Vec::new();

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

    pub fn create_vcpu(&mut self) -> Result<usize, Error> {
        // let _me = &self.me;
        let _vcpu = VCPU::new(self.me.clone());
        _vcpu.lock().set_vttbr(
            self.id as u64,
            self.page_table.lock().get_base_address() as u64,
        );

        self.vcpus.push(_vcpu);
        // self.vcpus.resize_with(vcpu, move || VCPU::new(_me.clone()));

        //         self.vcpus.iter().for_each(|vcpu: &Arc<Mutex<VCPU<T>>>| {
        //             vcpu.lock().set_vttbr(
        //                 self.id as u64,
        //                 self.page_table.lock().get_base_address() as u64,
        //             );
        //         });

        Ok(self.vcpus.len() - 1)
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
