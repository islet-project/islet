use crate::impl_addr;
use crate::mm::page::Address;
use core::fmt;

use core::ops::{Add, AddAssign, BitAnd, BitOr, Sub, SubAssign};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);
impl_addr!(VirtAddr);
