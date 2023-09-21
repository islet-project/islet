pub mod config;
pub mod context;
pub mod mm;
pub mod registry;
pub mod timer;
pub mod vcpu;

use crate::measurement::{Measurement, MEASUREMENTS_SLOT_NR};
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
    pub vmid: u16,
    pub state: State,
    pub vcpus: Vec<Arc<Mutex<VCPU<T>>>>,
    pub page_table: Arc<Mutex<Box<dyn IPATranslation>>>,
    pub measurements: [Measurement; MEASUREMENTS_SLOT_NR],
}

impl<T: Context + Default> Realm<T> {
    pub fn new(
        id: usize,
        vmid: u16,
        page_table: Arc<Mutex<Box<dyn IPATranslation>>>,
    ) -> Arc<Mutex<Self>> {
        Arc::new({
            let vcpus = Vec::new();
            let realm = Mutex::new(Self {
                id,
                vmid,
                state: State::New,
                vcpus: vcpus,
                page_table: page_table,
                measurements: [Measurement::empty(); MEASUREMENTS_SLOT_NR],
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
    Null,
    New,
    Active,
    SystemOff,
}
