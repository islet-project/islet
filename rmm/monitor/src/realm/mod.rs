pub mod mm;
pub mod vcpu;

use crate::realm::mm::IPATranslation;
use crate::realm::vcpu::{Context, VCPU};

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
