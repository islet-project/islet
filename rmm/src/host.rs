use crate::granule::validate_addr;
#[cfg(feature = "gst_page_table")]
use crate::granule::{is_not_in_realm, GRANULE_SIZE};
#[cfg(not(feature = "gst_page_table"))]
use crate::granule::{GranuleState, GRANULE_SIZE};
use crate::mm::translation::PageTable;
#[cfg(not(feature = "gst_page_table"))]
use crate::{get_granule, get_granule_if};

use safe_abstraction::raw_ptr::{assume_safe, SafetyAssured, SafetyChecked};
use vmsa::guard::Content;

pub fn copy_from<T: SafetyChecked + SafetyAssured + Copy>(addr: usize) -> Option<T> {
    #[cfg(feature = "gst_page_table")]
    if !validate_addr(addr) || !is_not_in_realm(addr) {
        return None;
    }
    #[cfg(not(feature = "gst_page_table"))]
    if !validate_addr(addr) || get_granule_if!(addr, GranuleState::Undelegated).is_err() {
        return None;
    }

    PageTable::get_ref().map(addr, false);
    let ret = assume_safe::<T>(addr).map(|safety_assumed| *safety_assumed);
    PageTable::get_ref().unmap(addr);

    match ret {
        Ok(obj) => Some(obj),
        Err(err) => {
            error!("Failed to convert a raw pointer to the struct. {:?}", err);
            None
        }
    }
}

pub fn copy_to_obj<T: SafetyChecked + SafetyAssured + Copy>(src: usize, dst: &mut T) -> Option<()> {
    #[cfg(feature = "gst_page_table")]
    if !validate_addr(src) || !is_not_in_realm(src) {
        return None;
    }
    #[cfg(not(feature = "gst_page_table"))]
    if !validate_addr(src) || get_granule_if!(src, GranuleState::Undelegated).is_err() {
        return None;
    }

    PageTable::get_ref().map(src, false);
    let ret = assume_safe::<T>(src).map(|safety_assumed| *dst = *safety_assumed);
    PageTable::get_ref().unmap(src);

    match ret {
        Ok(_) => Some(()),
        Err(err) => {
            error!("Failed to convert a raw pointer to the struct. {:?}", err);
            None
        }
    }
}

pub fn copy_to_ptr<T: SafetyChecked + SafetyAssured + Copy>(src: &T, dst: usize) -> Option<()> {
    #[cfg(feature = "gst_page_table")]
    if !validate_addr(dst) || !is_not_in_realm(dst) {
        return None;
    }
    #[cfg(not(feature = "gst_page_table"))]
    if !validate_addr(dst) || get_granule_if!(dst, GranuleState::Undelegated).is_err() {
        return None;
    }

    PageTable::get_ref().map(dst, false);
    let ret = assume_safe::<T>(dst).map(|mut safety_assumed| *safety_assumed = *src);
    PageTable::get_ref().unmap(dst);

    match ret {
        Ok(_) => Some(()),
        Err(err) => {
            error!("Failed to convert a raw pointer to the struct. {:?}", err);
            None
        }
    }
}

/// DataPage is used to convey realm data from host to realm.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataPage([u8; GRANULE_SIZE]);

impl DataPage {
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Default for DataPage {
    fn default() -> Self {
        Self([0; GRANULE_SIZE])
    }
}

impl Content for DataPage {}

impl safe_abstraction::raw_ptr::RawPtr for DataPage {}

impl safe_abstraction::raw_ptr::SafetyChecked for DataPage {}

impl safe_abstraction::raw_ptr::SafetyAssured for DataPage {
    fn is_initialized(&self) -> bool {
        // Given the fact that this memory is initialized by the Host,
        // it's not possible to unequivocally guarantee
        // that the values have been initialized from the perspective of the RMM.
        // However, any values, whether correctly initialized or not, will undergo
        // verification during the Measurement phase.
        // Consequently, this function returns `true`.
        true
    }

    fn verify_ownership(&self) -> bool {
        // This memory has permissions from the Host's perspective,
        // which inherently implies that exclusive ownership cannot be guaranteed by the RMM alone.
        // However, since the RMM only performs read operations and any incorrect values will be
        // verified during the Measurement phase.
        // Consequently, this function returns `true`.
        true
    }
}
