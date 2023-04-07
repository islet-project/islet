use core::fmt;
use monitor::impl_addr;
use monitor::mm::page::Address;

use core::ops::{Add, AddAssign, BitAnd, BitOr, Sub, SubAssign};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);
impl_addr!(VirtAddr);
