use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitOr, Sub, SubAssign};

use paging::address::Address;
use paging::impl_addr;

pub use paging::address::PhysAddr;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuestPhysAddr(usize);

impl_addr!(GuestPhysAddr);
