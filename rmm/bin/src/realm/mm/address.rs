use core::convert::{From, Into};
use core::ops;

pub fn to_paddr(addr: usize) -> PhysAddr {
    PhysAddr(addr)
}

pub fn to_vaddr(addr: PhysAddr) -> usize {
    addr.as_usize()
}

/// Align address downwards.
///
/// Returns the greatest x with alignment `align` so that x <= addr.
/// The alignment must be a power of 2.
#[inline(always)]
fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest x with alignment `align` so that x >= addr.
/// The alignment must be a power of 2.
#[inline(always)]
fn align_up(addr: usize, align: usize) -> usize {
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        (addr | align_mask) + 1
    }
}

/// A wrapper for a physical address, which is in principle
/// derived from the crate x86.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysAddr(pub usize);

impl PhysAddr {
    /// Convert to `u64`
    pub fn as_u64(self) -> u64 {
        self.0 as u64
    }

    /// Convert to `usize`
    pub fn as_usize(self) -> usize {
        self.0
    }

    /// Physical Address zero.
    pub const fn zero() -> Self {
        PhysAddr(0)
    }

    /// Is zero?
    pub fn is_zero(self) -> bool {
        self == PhysAddr::zero()
    }

    fn align_up<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        PhysAddr(align_up(self.0, align.into()))
    }

    fn align_down<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        PhysAddr(align_down(self.0, align.into()))
    }

    /// Is this address aligned to `align`?
    ///
    /// # Note
    /// `align` must be a power of two.
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<usize> + Copy,
    {
        if !align.into().is_power_of_two() {
            return false;
        }

        self.align_down(align) == self
    }
}

impl From<usize> for PhysAddr {
    fn from(num: usize) -> Self {
        PhysAddr(num)
    }
}

impl From<u64> for PhysAddr {
    fn from(num: u64) -> Self {
        PhysAddr(num as usize)
    }
}

impl From<i32> for PhysAddr {
    fn from(num: i32) -> Self {
        PhysAddr(num as usize)
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for PhysAddr {
    fn into(self) -> usize {
        self.0
    }
}

#[allow(clippy::from_over_into)]
impl Into<u64> for PhysAddr {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl ops::Add for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr(self.0 + rhs.0)
    }
}

impl ops::Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        PhysAddr::from(self.0 + rhs)
    }
}

impl ops::Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::from(self.0 + rhs as usize)
    }
}

impl ops::AddAssign for PhysAddr {
    fn add_assign(&mut self, other: PhysAddr) {
        *self = PhysAddr::from(self.0 + other.0);
    }
}

impl ops::AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, offset: usize) {
        *self = PhysAddr::from(self.0 + offset);
    }
}

impl ops::Sub for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr::from(self.0 - rhs.0)
    }
}

impl ops::Sub<usize> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        PhysAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::from(self.0 - rhs as usize)
    }
}

impl ops::Rem for PhysAddr {
    type Output = PhysAddr;

    fn rem(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<usize> for PhysAddr {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::Rem<u64> for PhysAddr {
    type Output = usize;

    fn rem(self, rhs: u64) -> Self::Output {
        self.0 % (rhs as usize)
    }
}

impl ops::BitAnd for PhysAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        PhysAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<usize> for PhysAddr {
    type Output = usize;

    fn bitand(self, rhs: usize) -> Self::Output {
        Into::<usize>::into(self) & rhs
    }
}

impl ops::BitOr for PhysAddr {
    type Output = PhysAddr;

    fn bitor(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<usize> for PhysAddr {
    type Output = usize;

    fn bitor(self, rhs: usize) -> Self::Output {
        self.0 | rhs
    }
}

impl ops::Shr<usize> for PhysAddr {
    type Output = usize;

    fn shr(self, rhs: usize) -> Self::Output {
        self.0 >> rhs
    }
}

impl ops::Shl<usize> for PhysAddr {
    type Output = usize;

    fn shl(self, rhs: usize) -> Self::Output {
        self.0 << rhs
    }
}

impl ops::Shl<u64> for PhysAddr {
    type Output = usize;

    fn shl(self, rhs: u64) -> Self::Output {
        self.0 << rhs as usize
    }
}

impl ops::Shl<i32> for PhysAddr {
    type Output = usize;

    fn shl(self, rhs: i32) -> Self::Output {
        self.0 << rhs as usize
    }
}

/// A wrapper for a virtual address, which is in principle
/// derived from the crate x86.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct GuestPhysAddr(pub usize);

impl GuestPhysAddr {
    /// Convert from `u64`
    pub const fn from_u64(v: u64) -> Self {
        GuestPhysAddr(v as usize)
    }

    /// Convert from `usize`
    pub const fn from_usize(v: usize) -> Self {
        GuestPhysAddr(v)
    }

    /// Convert to `u64`
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }

    /// Convert to `usize`
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Convert to mutable pointer.
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Convert to pointer.
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Physical Address zero.
    pub const fn zero() -> Self {
        GuestPhysAddr(0)
    }

    /// Is zero?
    pub fn is_zero(self) -> bool {
        self == GuestPhysAddr::zero()
    }

    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        GuestPhysAddr(align_up(self.0, align.into()))
    }

    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        GuestPhysAddr(align_down(self.0, align.into()))
    }

    /// Offset within the 4 KiB page.
    pub fn base_page_offset(self) -> usize {
        self.0 & (BASE_PAGE_SIZE as usize - 1)
    }

    /// Offset within the 2 MiB page.
    pub fn large_page_offset(self) -> usize {
        self.0 & (LARGE_PAGE_SIZE as usize - 1)
    }

    /// Return address of nearest 4 KiB page (lower or equal than self).
    pub fn align_down_to_base_page(self) -> Self {
        self.align_down(BASE_PAGE_SIZE as usize)
    }

    /// Return address of nearest 2 MiB page (lower or equal than self).
    pub fn align_down_to_large_page(self) -> Self {
        self.align_down(LARGE_PAGE_SIZE as usize)
    }

    /// Return address of nearest 4 KiB page (higher or equal than self).
    pub fn align_up_to_base_page(self) -> Self {
        self.align_up(BASE_PAGE_SIZE as usize)
    }

    /// Return address of nearest 2 MiB page (higher or equal than self).
    pub fn align_up_to_large_page(self) -> Self {
        self.align_up(LARGE_PAGE_SIZE as usize)
    }

    /// Is this address aligned to a 4 KiB page?
    pub fn is_base_page_aligned(self) -> bool {
        self.align_down(BASE_PAGE_SIZE as usize) == self
    }

    /// Is this address aligned to a 2 MiB page?
    pub fn is_large_page_aligned(self) -> bool {
        self.align_down(LARGE_PAGE_SIZE as usize) == self
    }

    /// Is this address aligned to `align`?
    ///
    /// # Note
    /// `align` must be a power of two.
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<usize> + Copy,
    {
        if !align.into().is_power_of_two() {
            return false;
        }

        self.align_down(align) == self
    }
}

impl From<usize> for GuestPhysAddr {
    fn from(num: usize) -> Self {
        GuestPhysAddr(num)
    }
}

impl From<i32> for GuestPhysAddr {
    fn from(num: i32) -> Self {
        GuestPhysAddr(num as usize)
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for GuestPhysAddr {
    fn into(self) -> usize {
        self.0
    }
}

impl From<u64> for GuestPhysAddr {
    fn from(num: u64) -> Self {
        GuestPhysAddr(num as usize)
    }
}

#[allow(clippy::from_over_into)]
impl Into<u64> for GuestPhysAddr {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl ops::Add for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn add(self, rhs: GuestPhysAddr) -> Self::Output {
        GuestPhysAddr(self.0 + rhs.0)
    }
}

impl ops::Add<usize> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        GuestPhysAddr(self.0 + rhs)
    }
}

impl ops::Add<u64> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        GuestPhysAddr::from(self.0 + rhs as usize)
    }
}

impl ops::AddAssign for GuestPhysAddr {
    fn add_assign(&mut self, other: GuestPhysAddr) {
        *self = GuestPhysAddr::from(self.0 + other.0);
    }
}

impl ops::AddAssign<usize> for GuestPhysAddr {
    fn add_assign(&mut self, offset: usize) {
        *self = GuestPhysAddr::from(self.0 + offset);
    }
}

impl ops::AddAssign<u64> for GuestPhysAddr {
    fn add_assign(&mut self, offset: u64) {
        *self = GuestPhysAddr::from(self.0 + offset as usize);
    }
}

impl ops::Sub for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn sub(self, rhs: GuestPhysAddr) -> Self::Output {
        GuestPhysAddr::from(self.0 - rhs.0)
    }
}

impl ops::Sub<usize> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        GuestPhysAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<u64> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        GuestPhysAddr::from(self.0 - rhs as usize)
    }
}

impl ops::Rem for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn rem(self, rhs: GuestPhysAddr) -> Self::Output {
        GuestPhysAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<usize> for GuestPhysAddr {
    type Output = usize;

    fn rem(self, rhs: Self::Output) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::Rem<u64> for GuestPhysAddr {
    type Output = usize;

    fn rem(self, rhs: u64) -> Self::Output {
        self.0 % (rhs as usize)
    }
}

impl ops::BitAnd for GuestPhysAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        GuestPhysAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<usize> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn bitand(self, rhs: usize) -> Self::Output {
        GuestPhysAddr(self.0 & rhs)
    }
}

impl ops::BitAnd<u64> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn bitand(self, rhs: u64) -> Self::Output {
        GuestPhysAddr(self.0 & rhs as usize)
    }
}

impl ops::BitAnd<i32> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn bitand(self, rhs: i32) -> Self::Output {
        GuestPhysAddr(self.0 & rhs as usize)
    }
}

impl ops::BitOr for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn bitor(self, rhs: GuestPhysAddr) -> GuestPhysAddr {
        GuestPhysAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<usize> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn bitor(self, rhs: usize) -> Self::Output {
        GuestPhysAddr(self.0 | rhs)
    }
}

impl ops::BitOr<u64> for GuestPhysAddr {
    type Output = GuestPhysAddr;

    fn bitor(self, rhs: u64) -> Self::Output {
        GuestPhysAddr(self.0 | rhs as usize)
    }
}

impl ops::Shr<usize> for GuestPhysAddr {
    type Output = usize;

    fn shr(self, rhs: usize) -> Self::Output {
        self.0 >> rhs
    }
}

impl ops::Shr<u64> for GuestPhysAddr {
    type Output = usize;

    fn shr(self, rhs: u64) -> Self::Output {
        self.0 >> rhs as usize
    }
}

impl ops::Shr<i32> for GuestPhysAddr {
    type Output = usize;

    fn shr(self, rhs: i32) -> Self::Output {
        self.0 >> rhs as usize
    }
}

impl ops::Shl<usize> for GuestPhysAddr {
    type Output = usize;

    fn shl(self, rhs: usize) -> Self::Output {
        self.0 << rhs as usize
    }
}

impl ops::Shl<u64> for GuestPhysAddr {
    type Output = usize;

    fn shl(self, rhs: u64) -> Self::Output {
        self.0 << rhs as usize
    }
}

impl ops::Shl<i32> for GuestPhysAddr {
    type Output = usize;

    fn shl(self, rhs: i32) -> Self::Output {
        self.0 << rhs as usize
    }
}

/// Log2 of base page size (12 bits).
pub const BASE_PAGE_SHIFT: usize = 12;

/// Size of a base page (4 KiB)
pub const BASE_PAGE_SIZE: usize = 4096;

/// Size of a large page (2 MiB)
pub const LARGE_PAGE_SIZE: usize = 1024 * 1024 * 2;
