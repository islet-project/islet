use crate::host::Accessor as HostAccessor;
use crate::rmm::PageMap;
use core::ops::Deref;

/// Type for holding an immutable pointer to physical region allocated by the host
#[repr(C)]
pub struct Pointer<T: HostAccessor> {
    /// pointer to phyiscal region
    ptr: *const T,
    /// page_map to map or unmap `ptr` in RMM
    page_map: PageMap,
}

impl<T: HostAccessor> Pointer<T> {
    /// Creates a new pointer pointing to data shared between the host and RMM
    pub fn new(ptr: usize, page_map: PageMap) -> Self {
        Self {
            ptr: ptr as *const T,
            page_map,
        }
    }

    /// Checks if this pointer is valid. It goes through two validations.
    ///   (1) T::acquire(): this function is used to validate page-relevant stuff (e.g., RMM map/unmap, GranuleState)
    ///   (2) T::validate():  this function is used to validate each field in T. (e.g., a constraint on parameter value)
    /// It returns a guard object only if it passes the two steps.
    #[inline]
    pub fn acquire<'a>(&'a self) -> Option<PointerGuard<'a, T>> {
        if T::acquire(self.ptr as usize, self.page_map) {
            let guard = PointerGuard { inner: self };
            if !guard.validate() {
                None
            } else {
                Some(guard)
            }
        } else {
            None
        }
    }
}

/// Guard for `Pointer`
pub struct PointerGuard<'a, T: HostAccessor> {
    inner: &'a Pointer<T>,
}

impl<'a, T: HostAccessor> PointerGuard<'a, T> {
    fn validate(&self) -> bool {
        // TODO: at this point, not sure we need this per-field validation.
        // we need to revisit this function after investigating RMM spec and TF-RMM once again.
        let obj = unsafe { &*self.inner.ptr };
        obj.validate()
    }
}

impl<'a, T: HostAccessor> Deref for PointerGuard<'a, T> {
    type Target = T;

    /// Safety: this is safe because
    /// the only safe way to get this `PointerGuard` is through `Pointer::acquire` method,
    /// and after the validation, it is safe to dereference the original pointer.
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.ptr }
    }
}

impl<'a, T: HostAccessor> Drop for PointerGuard<'a, T> {
    /// Automatically clean up page-relevant stuff we did in `acquire()`.
    fn drop(&mut self) {
        T::release(self.inner.ptr as usize, self.inner.page_map);
    }
}
