use crate::rmi::error::Error;
use core::ffi::c_void;
use core::fmt::Debug;

pub mod address;

use address::{GuestPhysAddr, PhysAddr};

pub trait IPATranslation: Debug + Send + Sync {
    fn get_base_address(&self) -> *const c_void;
    fn set_pages(
        &mut self,
        guest: GuestPhysAddr,
        phys: PhysAddr,
        size: usize,
        flags: usize,
        is_raw: bool,
    ) -> Result<(), Error>;
    fn unset_pages(&mut self, guest: GuestPhysAddr, size: usize);
    // TODO: remove mut
    fn ipa_to_pa(&mut self, guest: GuestPhysAddr, level: usize) -> Option<PhysAddr>;
    fn ipa_to_pte(&mut self, guest: GuestPhysAddr, level: usize) -> Option<(u64, usize)>;
    fn clean(&mut self);
}
