use core::fmt::Debug;

pub mod address;

use address::{GuestPhysAddr, PhysAddr};

pub trait IPATranslation: Debug + Send + Sync {
    fn get_vttbr(&self, vmid: usize) -> u64;
    fn set_pages(&mut self, guest: GuestPhysAddr, phys: PhysAddr, size: usize, flags: usize);
    fn unset_pages(&mut self, guest: GuestPhysAddr, size: usize);
}
