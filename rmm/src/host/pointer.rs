use crate::host::Accessor as HostAccessor;
use core::ops::{Deref, DerefMut};

/// Type for holding an immutable pointer to physical region allocated by the host
#[repr(C)]
pub struct Pointer<T: HostAccessor> {
    /// pointer to physical region
    ptr: *const T,
}

impl<T: HostAccessor> Pointer<T> {
    /// Creates a new pointer pointing to data shared between the host and RMM
    pub fn new(ptr: usize) -> Self {
        Self {
            ptr: ptr as *const T,
        }
    }

    /// Checks if this pointer is valid. It goes through two validations.
    ///   (1) T::acquire(): this function is used to validate page-relevant stuff (e.g., RMM map/unmap, GranuleState)
    ///   (2) T::validate():  this function is used to validate each field in T. (e.g., a constraint on parameter value)
    /// It returns a guard object only if it passes the two steps.
    #[inline]
    pub fn acquire(&self) -> Option<PointerGuard<'_, T>> {
        if T::acquire(self.ptr as usize) {
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
        T::release(self.inner.ptr as usize);
    }
}

/// Type for holding a mutable pointer to physical region allocated by the host
#[repr(C)]
pub struct PointerMut<T: HostAccessor> {
    /// pointer to phyiscal region
    ptr: *mut T,
}

impl<T: HostAccessor> PointerMut<T> {
    pub fn new(ptr: usize) -> Self {
        Self { ptr: ptr as *mut T }
    }

    #[inline]
    pub fn acquire(&mut self) -> Option<PointerMutGuard<'_, T>> {
        if T::acquire(self.ptr as usize) {
            let guard = PointerMutGuard { inner: self };
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

pub struct PointerMutGuard<'a, T: HostAccessor> {
    inner: &'a PointerMut<T>,
}

impl<'a, T: HostAccessor> PointerMutGuard<'a, T> {
    fn validate(&self) -> bool {
        let obj = unsafe { &*self.inner.ptr };
        obj.validate()
    }
}

impl<'a, T: HostAccessor> Deref for PointerMutGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.ptr }
    }
}

impl<'a, T: HostAccessor> DerefMut for PointerMutGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.ptr }
    }
}

impl<'a, T: HostAccessor> Drop for PointerMutGuard<'a, T> {
    fn drop(&mut self) {
        T::release(self.inner.ptr as usize);
    }
}

#[macro_export]
macro_rules! copy_from_host_or_ret {
    ($target_type:tt, $ptr:expr) => {{
        use crate::granule::is_not_in_realm;
        use crate::rmi::error::Error;

        if !is_not_in_realm($ptr) {
            return Err(Error::RmiErrorInput);
        }

        let src_obj = HostPointer::<$target_type>::new($ptr);
        let src_obj = src_obj.acquire();
        let src_obj = if let Some(v) = src_obj {
            v
        } else {
            return Err(Error::RmiErrorInput);
        };

        let mut dst_obj: $target_type = $target_type::default();
        unsafe {
            core::ptr::copy_nonoverlapping::<$target_type>(
                &*src_obj as *const $target_type,
                &mut dst_obj as *mut $target_type,
                1,
            );
        };
        dst_obj
    }};
    ($target_type:tt, $ptr:expr, $code:expr) => {{
        use crate::granule::is_not_in_realm;
        use crate::rmi::error::Error;

        if !is_not_in_realm($ptr) {
            return Err(Error::RmiErrorInput);
        }

        let src_obj = HostPointer::<$target_type>::new($ptr);
        let src_obj = src_obj.acquire();
        let src_obj = if let Some(v) = src_obj {
            v
        } else {
            return Err($code);
        };

        let mut dst_obj: $target_type = $target_type::default();
        unsafe {
            core::ptr::copy_nonoverlapping::<$target_type>(
                &*src_obj as *const $target_type,
                &mut dst_obj as *mut $target_type,
                1,
            );
        };
        dst_obj
    }};
}

#[macro_export]
macro_rules! copy_to_host_or_ret {
    ($target_type:tt, $src_obj:expr, $dest_ptr:expr) => {{
        let mut dst_obj = HostPointerMut::<$target_type>::new($dest_ptr);
        let dst_obj = dst_obj.acquire();
        let mut dst_obj = if let Some(v) = dst_obj {
            v
        } else {
            use crate::rmi::error::Error;
            return Err(Error::RmiErrorInput);
        };

        unsafe {
            core::ptr::copy_nonoverlapping::<$target_type>(
                $src_obj as *const $target_type,
                &mut *dst_obj as *mut $target_type,
                1,
            );
        };
    }};
}
