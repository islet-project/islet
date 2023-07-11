pub mod pointer;

use crate::rmm::PageMap;

/// This trait is used to enforce security checks for physical region allocated by the host.
pub trait Accessor {
    /// Try to do page-relevant stuff (e.g., RMM map).
    /// returns true only if everything goes well.
    fn acquire(ptr: usize, page_map: PageMap) -> bool {
        page_map.map(ptr, false)
    }

    /// Try to clean up page-relevant stuff done by `acquire`.
    /// Structs that implement this trait must synchronize this function with `acquire`.
    /// returns true only if everything goes well.
    fn release(ptr: usize, page_map: PageMap) -> bool {
        page_map.unmap(ptr)
    }

    /// Validate each field in a struct that implements this trait.
    /// returns true only if everything goes well.
    fn validate(&self) -> bool {
        true
    }
}
