use core::ffi::c_void;
use core::fmt::Debug;

pub mod address;

use address::{GuestPhysAddr, PhysAddr};

pub trait IPATranslation: Debug + Send + Sync {
    fn get_base_address(&self) -> *const c_void;
    fn set_pages(&mut self, guest: GuestPhysAddr, phys: PhysAddr, size: usize, flags: usize);
    fn unset_pages(&mut self, guest: GuestPhysAddr, size: usize);
    fn clean(&mut self);
}
