pub mod mm;
pub mod vcpu;

use crate::realm::mm::IPATranslation;
use crate::realm::vcpu::{Context, VCPU};

use crate::error::Error;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Debug;
use spin::mutex::Mutex;

extern crate alloc;

#[derive(Debug)]
pub struct Realm<T: Context> {
    id: usize,
    pub state: State,
    pub vcpus: Vec<Arc<Mutex<VCPU<T>>>>,
    pub page_table: Arc<Mutex<Box<dyn IPATranslation>>>,
}

impl<T: Context + Default> Realm<T> {
    pub fn new(id: usize, page_table: Arc<Mutex<Box<dyn IPATranslation>>>) -> Arc<Mutex<Self>> {
        Arc::new({
            let vcpus = Vec::new();
            let realm = Mutex::new(Self {
                id: id,
                state: State::Init,
                vcpus: vcpus,
                page_table: page_table,
            });
            realm
        })
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
