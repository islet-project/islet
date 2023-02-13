pub mod mm;
pub mod vcpu;

use crate::realm::mm::IPATranslation;
use crate::realm::vcpu::{Context, VCPU};

use crate::error::{Error, ErrorKind};
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::fmt::Debug;
use spin::mutex::Mutex;

extern crate alloc;

#[derive(Debug)]
pub struct Realm<T: Context> {
    id: usize,
    me: Weak<Mutex<Self>>,
    pub state: State,
    pub vcpus: Vec<Arc<Mutex<VCPU<T>>>>,
    pub page_table: Arc<Mutex<Box<dyn IPATranslation>>>,
}

impl<T: Context + Default> Realm<T> {
    pub fn new(id: usize, page_table: Arc<Mutex<Box<dyn IPATranslation>>>) -> Arc<Mutex<Self>> {
        Arc::<Mutex<Self>>::new_cyclic(|me| {
            let vcpus = Vec::new();
            let realm = Mutex::new(Self {
                id: id,
                me: me.clone(),
                state: State::Init,
                vcpus: vcpus,
                page_table: page_table,
            });
            realm
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

impl<T: Context> Drop for Realm<T> {
    fn drop(&mut self) {
        info!("Realm #{} was destroyed!", self.id);
    }
}

#[derive(Debug)]
pub enum State {
    Init,
    Ready,
    Running,
    Destroy,
}

pub type Manager = &'static dyn Control;
static mut REALM: Option<Manager> = None;

#[allow(unused_must_use)]
pub fn set_instance(vm: Manager) {
    unsafe {
        if REALM.is_none() {
            REALM = Some(vm);
        }
    };
}

pub fn instance() -> Option<Manager> {
    unsafe { REALM }
}

pub trait Control: Debug + Send + Sync {
    fn create(&self) -> Result<usize, &str>;
    fn create_vcpu(&self, id: usize) -> Result<usize, Error>;
    fn remove(&self, id: usize) -> Result<(), &str>;
    fn run(&self, id: usize, vcpu: usize, incr_pc: usize) -> Result<([usize; 4]), &str>;
    fn map(
        &self,
        id: usize,
        guest: usize,
        phys: usize,
        size: usize,
        prot: usize,
    ) -> Result<(), &str>;
    fn unmap(&self, id: usize, guest: usize, size: usize) -> Result<(), &str>;
    fn set_reg(&self, id: usize, vcpu: usize, register: usize, value: usize) -> Result<(), &str>;
    fn get_reg(&self, id: usize, vcpu: usize, register: usize) -> Result<usize, &str>;
}
