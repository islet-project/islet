use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitOr, Sub, SubAssign};

use vmsa::address::Address;
use vmsa::impl_addr;

pub use vmsa::address::PhysAddr;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuestPhysAddr(usize);

impl_addr!(GuestPhysAddr);
