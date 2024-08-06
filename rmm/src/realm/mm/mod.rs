pub mod address;
pub mod attribute;
pub mod entry;
pub mod page;
pub mod rtt;
pub mod stage2_translation;
pub mod stage2_tte;
pub mod table_level;

use crate::rmi::error::Error;
use core::ffi::c_void;
use core::fmt::Debug;
use core::slice::Iter;

use address::{GuestPhysAddr, PhysAddr};
use stage2_translation::Tlbi;

pub trait IPATranslation: Debug + Send + Sync {
    fn get_base_address(&self) -> *const c_void;
    // TODO: remove mut
    fn ipa_to_pa(&mut self, guest: GuestPhysAddr, level: usize) -> Option<PhysAddr>;
    fn ipa_to_pte(&mut self, guest: GuestPhysAddr, level: usize) -> Option<(u64, usize)>;
    fn ipa_to_pte_set(
        &mut self,
        guest: GuestPhysAddr,
        level: usize,
        val: u64,
        invalidate: Tlbi,
    ) -> Result<(), Error>;
    fn clean(&mut self, vmid: usize);
    fn space_size(&self, level: usize) -> usize;
    fn entries(
        &self,
        guest: GuestPhysAddr,
        level: usize,
    ) -> Result<(Iter<'_, entry::Entry>, usize), Error>;
}
