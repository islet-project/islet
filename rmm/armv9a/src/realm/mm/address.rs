use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitOr, Sub, SubAssign};

/// Align address downwards.
///
/// Returns the greatest x with alignment `align` so that x <= addr.
/// The alignment must be a power of 2.
#[inline(always)]
pub fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest x with alignment `align` so that x >= addr.
/// The alignment must be a power of 2.
#[inline(always)]
pub fn align_up(addr: usize, align: usize) -> usize {
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        (addr | align_mask) + 1
    }
}

/// A wrapper for a physical address
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(usize);

/// A wrapper for a virtual address
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuestPhysAddr(usize);

macro_rules! impl_addr {
    ($T:tt) => {
        impl Add for $T {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                $T(self.0 + other.0)
            }
        }

        impl AddAssign for $T {
            fn add_assign(&mut self, other: Self) {
                self.0 = self.0 + other.0;
            }
        }

        impl Sub for $T {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                $T(self.0 - other.0)
            }
        }

        impl SubAssign for $T {
            fn sub_assign(&mut self, other: Self) {
                self.0 = self.0 - other.0;
            }
        }

        impl BitAnd for $T {
            type Output = Self;
            fn bitand(self, other: Self) -> Self {
                $T(self.0 & other.0)
            }
        }

        impl BitOr for $T {
            type Output = Self;
            fn bitor(self, other: Self) -> Self {
                $T(self.0 | other.0)
            }
        }

        impl<T: Sized> From<*mut T> for $T {
            fn from(raw_ptr: *mut T) -> $T {
                $T(raw_ptr as usize)
            }
        }

        impl<T: Sized> From<*const T> for $T {
            fn from(raw_ptr: *const T) -> $T {
                $T(raw_ptr as usize)
            }
        }

        impl From<usize> for $T {
            fn from(raw_ptr: usize) -> Self {
                $T(raw_ptr)
            }
        }

        impl From<u64> for $T {
            fn from(raw_ptr: u64) -> Self {
                $T(raw_ptr as usize)
            }
        }

        impl From<i32> for $T {
            fn from(raw_ptr: i32) -> Self {
                $T(raw_ptr as usize)
            }
        }

        impl $T {
            pub fn as_u64(&self) -> u64 {
                self.0 as u64
            }

            pub fn as_usize(&self) -> usize {
                self.0
            }

            pub const fn zero() -> Self {
                $T(0)
            }
        }

        impl fmt::Debug for $T {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($T))
                    .field("{:#016x}", &self.0)
                    .finish()
            }
        }
    };
}

impl_addr!(PhysAddr);
impl_addr!(GuestPhysAddr);
