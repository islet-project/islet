//! Bump allocator over a contiguous virtual address range mapped to REC AUX granules.
//!
//! Allocations happen once during `RMI_REC_CREATE` and the entire backing
//! region is released all at once during `RMI_REC_DESTROY` (or rollback).

use core::alloc::Layout;
use core::ptr::NonNull;
use vmsa::address::VirtAddr;

/// Bump (linear) allocator managing a contiguous virtual address range.
#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AuxBumpAllocator {
    /// Starting virtual address of the managed region (`0` when uninitialized).
    base: usize,
    /// Total capacity of the managed region in bytes.
    size: usize,
    /// Current bump offset in bytes from `base`.
    offset: usize,
}

impl AuxBumpAllocator {
    /// Creates an uninitialized bump allocator with zero capacity.
    pub const fn new() -> Self {
        Self {
            base: 0,
            size: 0,
            offset: 0,
        }
    }

    /// Initializes the bump allocator with a mapped virtual address range `[va, va + size)`.
    pub fn init(&mut self, va: VirtAddr, size: usize) {
        self.base = va.as_usize();
        self.size = size;
        self.offset = 0;
    }

    /// Resets the allocator back to its uninitialized state.
    pub fn reset(&mut self) {
        self.base = 0;
        self.size = 0;
        self.offset = 0;
    }

    /// Allocates a block of memory satisfying `layout` by bumping the cursor.
    ///
    /// Returns `None` if the allocator is uninitialized or does not have
    /// enough remaining space to satisfy `layout`.
    pub fn alloc_layout(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        if self.base == 0 || self.size == 0 {
            return None;
        }

        let current = self.base.checked_add(self.offset)?;
        let align_mask = layout.align() - 1;
        let aligned_start = current.checked_add(align_mask)? & !align_mask;
        let new_offset = aligned_start
            .checked_sub(self.base)?
            .checked_add(layout.size())?;

        if new_offset > self.size {
            return None;
        }

        self.offset = new_offset;
        NonNull::new(aligned_start as *mut u8)
    }

    /// Allocates uninitialized space for a value of type `T`.
    pub fn alloc<T>(&mut self) -> Option<NonNull<T>> {
        self.alloc_layout(Layout::new::<T>()).map(NonNull::cast)
    }

    /// Returns the base virtual address of the managed region.
    pub fn base(&self) -> VirtAddr {
        VirtAddr::from(self.base)
    }

    /// Returns the total capacity of the managed region in bytes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the number of bytes consumed so far (including alignment padding).
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Returns the number of unallocated bytes remaining at the tail of the region.
    pub fn remaining(&self) -> usize {
        self.size.saturating_sub(self.offset)
    }

    /// Returns `true` if the allocator has been initialized with a non-empty region.
    pub fn is_initialized(&self) -> bool {
        self.base != 0 && self.size != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C, align(16))]
    struct Aligned16([u8; 32]);

    #[test]
    fn test_aux_bump_allocator_basic_and_alignment() {
        let mut alloc = AuxBumpAllocator::new();
        assert!(!alloc.is_initialized());
        assert!(alloc.alloc::<u64>().is_none());

        alloc.init(VirtAddr::from(0x2000_0000usize), 128);
        assert!(alloc.is_initialized());
        assert_eq!(alloc.base(), VirtAddr::from(0x2000_0000usize));
        assert_eq!(alloc.size(), 128);
        assert_eq!(alloc.used(), 0);
        assert_eq!(alloc.remaining(), 128);

        // Allocate 1 byte at offset 0 -> used = 1
        let p1 = alloc.alloc::<u8>().unwrap();
        assert_eq!(p1.as_ptr() as usize, 0x2000_0000);
        assert_eq!(alloc.used(), 1);

        // Next allocation requires 16-byte alignment -> aligned to 0x2000_0010, size 32 -> used = 48
        let p2 = alloc.alloc::<Aligned16>().unwrap();
        assert_eq!(p2.as_ptr() as usize, 0x2000_0010);
        assert_eq!(alloc.used(), 48);
        assert_eq!(alloc.remaining(), 80);

        // Allocate 80 bytes -> exact fit (used = 128)
        let layout_80 = Layout::from_size_align(80, 8).unwrap();
        let p3 = alloc.alloc_layout(layout_80).unwrap();
        assert_eq!(p3.as_ptr() as usize, 0x2000_0030);
        assert_eq!(alloc.used(), 128);
        assert_eq!(alloc.remaining(), 0);

        // Region is full -> further non-zero allocations return None
        assert!(alloc.alloc::<u8>().is_none());

        alloc.reset();
        assert!(!alloc.is_initialized());
        assert_eq!(alloc.used(), 0);
    }

    #[test]
    fn test_aux_bump_allocator_uninitialized_and_zero_size() {
        let mut alloc = AuxBumpAllocator::new();
        assert!(!alloc.is_initialized());
        assert_eq!(alloc.base(), VirtAddr::from(0usize));
        assert_eq!(alloc.size(), 0);
        assert_eq!(alloc.used(), 0);
        assert_eq!(alloc.remaining(), 0);
        assert!(alloc.alloc::<u64>().is_none());

        // Zero base address must be treated as uninitialized
        alloc.init(VirtAddr::from(0usize), 4096);
        assert!(!alloc.is_initialized());
        assert!(alloc.alloc::<u64>().is_none());

        // Zero size must be treated as uninitialized
        alloc.init(VirtAddr::from(0x2000_0000usize), 0);
        assert!(!alloc.is_initialized());
        assert!(alloc.alloc::<u64>().is_none());
    }

    #[test]
    fn test_aux_bump_allocator_out_of_memory_and_overflow() {
        let mut alloc = AuxBumpAllocator::new();
        alloc.init(VirtAddr::from(0x2000_0000usize), 32);

        // Requesting more than capacity fails without advancing offset
        let too_large = Layout::from_size_align(64, 16).unwrap();
        assert!(alloc.alloc_layout(too_large).is_none());
        assert_eq!(alloc.used(), 0);

        // Advance offset by 24 bytes; a 16-byte-aligned 16-byte allocation now
        // needs offset 32..48, which exceeds size 32 -> returns None and preserves offset.
        let layout_24 = Layout::from_size_align(24, 8).unwrap();
        assert!(alloc.alloc_layout(layout_24).is_some());
        assert_eq!(alloc.used(), 24);
        let layout_16_aligned = Layout::from_size_align(16, 16).unwrap();
        assert!(alloc.alloc_layout(layout_16_aligned).is_none());
        assert_eq!(alloc.used(), 24);

        // Near-usize::MAX base address overflow check
        let mut high_alloc = AuxBumpAllocator::new();
        high_alloc.init(VirtAddr::from(usize::MAX - 7), 16);
        assert!(high_alloc.alloc::<Aligned16>().is_none());
    }

    #[test]
    fn test_aux_bump_allocator_reset_and_reinit() {
        let mut alloc = AuxBumpAllocator::new();
        alloc.init(VirtAddr::from(0x2000_0000usize), 64);

        let p1 = alloc.alloc::<u64>().unwrap();
        assert_eq!(p1.as_ptr() as usize, 0x2000_0000);
        assert_eq!(alloc.used(), 8);

        alloc.reset();
        assert!(!alloc.is_initialized());
        assert_eq!(alloc.base(), VirtAddr::from(0usize));
        assert_eq!(alloc.size(), 0);
        assert_eq!(alloc.used(), 0);
        assert_eq!(alloc.remaining(), 0);

        // Re-initializing with a new VA range starts fresh at offset 0
        alloc.init(VirtAddr::from(0x3000_0000usize), 4096);
        let p2 = alloc.alloc::<Aligned16>().unwrap();
        assert_eq!(p2.as_ptr() as usize, 0x3000_0000);
        assert_eq!(alloc.used(), 32);
        assert_eq!(alloc.remaining(), 4096 - 32);
    }
}
