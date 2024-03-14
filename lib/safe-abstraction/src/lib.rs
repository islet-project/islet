#![warn(rust_2018_idioms)]
#![deny(warnings)]
#![no_std]

pub trait RawPtr {
    /// # Safety
    ///
    /// When calling this method, you have to ensure that all of the following is true:
    ///
    /// * The pointer must point to an initialized instance of `T`.
    ///
    /// * You must enforce Rust's aliasing rules
    unsafe fn as_ref<'a, T: RawPtr>(addr: usize) -> Option<&'a T> {
        match Self::is_valid::<T>(addr) {
            true => Some(&*(addr as *const T)),
            false => None,
        }
    }

    /// # Safety
    ///
    /// When calling this method, you have to ensure that all of the following is true:
    ///
    /// * The pointer must point to an initialized instance of `T`.
    ///
    /// * You must enforce Rust's aliasing rules
    unsafe fn as_mut<'a, T: RawPtr>(addr: usize) -> Option<&'a mut T> {
        match Self::is_valid::<T>(addr) {
            true => Some(&mut *(addr as *mut T)),
            false => None,
        }
    }

    fn is_valid<T: RawPtr>(addr: usize) -> bool {
        let ptr = addr as *const T;
        // Safety: This cast from a raw pointer to a reference is considered safe
        //         because it is used solely for the purpose of verifying alignment and range,
        //         without actually dereferencing the pointer.
        let ref_ = unsafe { &*(ptr) };
        !ptr.is_null() && ref_.is_aligned() && ref_.is_within_range()
    }

    fn addr(&self) -> usize {
        let ptr: *const Self = self;
        ptr as usize
    }

    fn is_aligned(&self) -> bool {
        self.addr().is_power_of_two()
    }

    fn is_within_range(&self) -> bool;
}
