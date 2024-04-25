pub mod config;
pub mod context;
pub mod mm;
pub mod registry;
pub mod timer;
pub mod vcpu;

use crate::realm::vcpu::VCPU;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Debug;
use spin::mutex::Mutex;

extern crate alloc;

#[derive(Debug)]
pub struct Realm {
    id: usize,
    pub vmid: u16,
    pub vcpus: Vec<Arc<Mutex<VCPU>>>,
}

impl Realm {
    pub fn new(id: usize, vmid: u16) -> Arc<Mutex<Self>> {
        Arc::new({
            let vcpus = Vec::new();
            Mutex::new(Self { id, vmid, vcpus })
        })
    }
}

impl Drop for Realm {
    fn drop(&mut self) {
        info!("Realm #{} was destroyed!", self.id);
    }
}
