use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitOr, Sub, SubAssign};

use crate::impl_addr;
use crate::mm::page::Address;

pub use crate::mm::address::PhysAddr;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuestPhysAddr(usize);

impl_addr!(GuestPhysAddr);
