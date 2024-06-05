use core::ops::{Deref, DerefMut};
use spinning_top::SpinlockGuard;

use super::error::Error;
use safe_abstraction::raw_ptr;

/// EntryGuard provides a secure interface to access Entry while holding the corresponding lock.
/// Also, it is used as a means of accessing "content" placed at the address of Entry under the lock.
pub struct EntryGuard<'a, E> {
    /// inner type for Entry, which corresponds to Entry::Inner
    inner: SpinlockGuard<'a, E>,
    /// address that this Entry holds
    addr: usize,
    /// flags of Entry
    #[allow(dead_code)]
    flags: u64,
}

impl<'a, E> EntryGuard<'a, E> {
    pub fn new(inner: SpinlockGuard<'a, E>, addr: usize, flags: u64) -> Self {
        Self { inner, addr, flags }
    }

    /// content placed at the `addr`. (e.g., Rec, DataPage, ...)
    /// access to this content is protected under the entry-level lock that "inner" holds.
    /// T is a target struct that `addr` maps to.
    pub fn content<T>(&self) -> Result<raw_ptr::SafetyAssumed<T>, Error>
    where
        T: Content + raw_ptr::SafetyChecked + raw_ptr::SafetyAssured,
    {
        // Note: flag can be used here for validation checks.
        //  e.g., `if T::FLAGS != self.flags { error }`
        //        for example of Granule, T::FLAGS is Rd while self.flags at run-time is not Rd.
        raw_ptr::assume_safe::<T>(self.addr).or(Err(Error::MmErrorOthers))
    }

    pub fn content_mut<T>(&mut self) -> Result<raw_ptr::SafetyAssumed<T>, Error>
    where
        T: Content + raw_ptr::SafetyChecked + raw_ptr::SafetyAssured,
    {
        raw_ptr::assume_safe::<T>(self.addr).or(Err(Error::MmErrorOthers))
    }
}

impl<'a, E> Deref for EntryGuard<'a, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, E> DerefMut for EntryGuard<'a, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait Content {}
