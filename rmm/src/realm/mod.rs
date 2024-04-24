pub mod config;
pub mod context;
pub mod mm;
pub mod registry;
pub mod timer;
pub mod vcpu;

use crate::realm::vcpu::{Context, VCPU};

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Debug;
use spin::mutex::Mutex;

extern crate alloc;

#[derive(Debug)]
pub struct Realm<T: Context> {
    id: usize,
    pub vmid: u16,
    pub vcpus: Vec<Arc<Mutex<VCPU<T>>>>,
}

impl<T: Context + Default> Realm<T> {
    pub fn new(id: usize, vmid: u16) -> Arc<Mutex<Self>> {
        Arc::new({
            let vcpus = Vec::new();
            Mutex::new(Self { id, vmid, vcpus })
        })
    }
}

impl<T: Context> Drop for Realm<T> {
    fn drop(&mut self) {
        info!("Realm #{} was destroyed!", self.id);
    }
}
