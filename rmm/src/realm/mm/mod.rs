pub mod address;
pub mod page;
pub mod page_table;
pub mod stage2_translation;
pub mod stage2_tte;
pub mod translation_granule_4k;

use crate::rmi::error::Error;
use core::ffi::c_void;
use core::fmt::Debug;

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
    fn ipa_to_pte_set(&mut self, guest: GuestPhysAddr, level: usize, val: u64)
        -> Result<(), Error>;
    fn clean(&mut self);
}
