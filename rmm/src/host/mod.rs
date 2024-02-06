#[macro_use]
pub mod pointer;

use crate::granule::validate_addr;
use crate::granule::GRANULE_SIZE;
use crate::mm::translation::PageTable;

use vmsa::guard::Content;

/// This trait is used to enforce security checks for physical region allocated by the host.
/// This is used for `PointerGuard` which is not able to modify data.
pub trait Accessor {
    /// Try to do page-relevant stuff (e.g., RMM map).
    /// returns true only if everything goes well.
    fn acquire(ptr: usize) -> bool {
        if !validate_addr(ptr) {
            return false;
        }
        // TODO: check if the granule state of `ptr` is Undelegated.
        PageTable::get_ref().map(ptr, false)
    }

    /// Try to clean up page-relevant stuff done by `acquire`.
    /// Structs that implement this trait must synchronize this function with `acquire`.
    /// returns true only if everything goes well.
    fn release(ptr: usize) -> bool {
        // TODO: check if the granule state of `ptr` is Undelegated.
        PageTable::get_ref().unmap(ptr)
    }

    /// Validate each field in a struct that implements this trait.
    /// returns true only if everything goes well.
    fn validate(&self) -> bool {
        true
    }
}

/// DataPage is used to convey realm data from host to realm.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataPage([u8; GRANULE_SIZE]);

impl DataPage {
    pub unsafe fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_ptr() as *mut u8
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Default for DataPage {
    fn default() -> Self {
        Self([0; GRANULE_SIZE])
    }
}

impl Accessor for DataPage {}

impl Content for DataPage {}
