//! Virtual Address Range allocator
//!
//! This module provides a virtual address space allocator that allows mapping
//! multiple discontiguous physical AUX granules to a contiguous virtual address range.

use crate::config::PAGE_SIZE;
use alloc::collections::BTreeMap;
use core::ops::Range;
use spin::mutex::Mutex;
use vmsa::address::VirtAddr;

/// Represents a contiguous virtual address range allocated for a multi-granule mapping
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiGranuleMapping {
    /// Virtual address range for this mapping
    pub va_range: Range<VirtAddr>,
    /// Number of pages mapped in this range
    pub num_pages: usize,
}

/// Virtual address range allocator for multi-granule mappings
pub struct VaRangeAllocator {
    /// Base virtual address for the VA range
    base: VirtAddr,
    /// Size of the VA range in bytes
    size: usize,
    /// Active mappings: va_start -> mapping
    mappings: BTreeMap<VirtAddr, MultiGranuleMapping>,
}

impl VaRangeAllocator {
    /// Create a new VA range allocator with the given base address and size
    pub const fn new(base: VirtAddr, size: usize) -> Self {
        Self {
            base,
            size,
            mappings: BTreeMap::new(),
        }
    }

    /// Allocate and register a contiguous virtual address range for the given
    /// number of pages using the first-fit algorithm.
    /// Returns the starting virtual address, or None if allocation fails.
    pub fn alloc(&mut self, num_pages: usize) -> Option<VirtAddr> {
        if num_pages == 0 {
            return None;
        }

        let size_needed = num_pages.checked_mul(PAGE_SIZE)?;
        if size_needed > self.size {
            return None;
        }

        let va_start = self.find_gap(size_needed)?;
        let va_end = va_start + VirtAddr::from(size_needed);

        self.mappings.insert(
            va_start,
            MultiGranuleMapping {
                va_range: va_start..va_end,
                num_pages,
            },
        );

        Some(va_start)
    }

    /// Find the first gap in existing mappings that can fit `size_needed` bytes
    fn find_gap(&self, size_needed: usize) -> Option<VirtAddr> {
        let mut search_start = self.base.as_usize();
        let pool_end = self.base.as_usize().checked_add(self.size)?;

        for (&va_start, mapping) in &self.mappings {
            let gap_end = va_start.as_usize();

            if gap_end >= search_start && gap_end - search_start >= size_needed {
                return Some(VirtAddr::from(search_start));
            }

            search_start = mapping.va_range.end.as_usize();
        }

        // Check the gap after the last mapping
        if pool_end >= search_start && pool_end - search_start >= size_needed {
            return Some(VirtAddr::from(search_start));
        }

        None
    }

    /// Remove a mapping by its exact starting virtual address
    pub fn remove(&mut self, va_start: VirtAddr) -> Option<MultiGranuleMapping> {
        self.mappings.remove(&va_start)
    }

    /// Get a mapping by its exact starting virtual address
    pub fn get(&self, va_start: VirtAddr) -> Option<&MultiGranuleMapping> {
        self.mappings.get(&va_start)
    }

    /// Get the base address of the pool
    pub fn base(&self) -> VirtAddr {
        self.base
    }

    /// Get the size of the pool
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the total currently allocated size in bytes
    pub fn used(&self) -> usize {
        self.mappings
            .values()
            .map(|m| m.num_pages * PAGE_SIZE)
            .sum()
    }

    /// Get the number of active mappings
    pub fn num_mappings(&self) -> usize {
        self.mappings.len()
    }

    /// Check if a virtual address is within this pool
    pub fn contains(&self, va: VirtAddr) -> bool {
        va >= self.base && va < self.base + VirtAddr::from(self.size)
    }
}

lazy_static! {
    /// Global VA range for multi-granule mappings
    /// This is initialized with a reserved region of virtual address space
    pub static ref VA_MULTI_GRANULE_ALLOCATOR: Mutex<VaRangeAllocator> = Mutex::new(VaRangeAllocator::new(
        VirtAddr::from(VA_MULTI_GRANULE_BASE),
        VA_MULTI_GRANULE_SIZE
    ));
}

/// Base address for the VA range
/// This should be in a region of virtual address space that doesn't conflict
/// with the identity-mapped RMM code/data or other mappings for VFP and Qemu platforms.
/// We place it low in the VA space.
pub const VA_MULTI_GRANULE_BASE: usize = 0x2000_0000;

/// Size of the VA range (64MB by default)
/// Per-realm: max 16 granules × 4KB × 8 vCPUs = 512KB
/// Max 64 realms: 32MB
/// Reserved: 64MB (2× headroom for future growth)
pub const VA_MULTI_GRANULE_SIZE: usize = 64 * 1024 * 1024;

/// Allocate a contiguous virtual address range for `num_pages` pages.
/// Returns the starting virtual address, or None if allocation fails.
pub fn alloc_multi_granule_mapping(num_pages: usize) -> Option<VirtAddr> {
    let mut allocator = VA_MULTI_GRANULE_ALLOCATOR.lock();
    allocator.alloc(num_pages)
}

/// Free a multi-granule mapping by its starting virtual address
pub fn free_multi_granule_mapping(va_start: VirtAddr) -> Option<MultiGranuleMapping> {
    let mut allocator = VA_MULTI_GRANULE_ALLOCATOR.lock();
    allocator.remove(va_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_va_range_allocator_alloc_and_used() {
        let mut allocator = VaRangeAllocator::new(VirtAddr::from(0x1000_0000usize), 0x1000_0000); // 16MB at 256MB

        // Zero pages should fail
        assert_eq!(allocator.alloc(0), None);

        // Overflowing page count should fail
        assert_eq!(allocator.alloc(usize::MAX), None);

        // Allocate 4 pages
        let va1 = allocator.alloc(4);
        assert_eq!(va1, Some(VirtAddr::from(0x1000_0000usize)));

        // Allocate another 2 pages
        let va2 = allocator.alloc(2);
        assert_eq!(va2, Some(VirtAddr::from(0x1000_4000usize)));

        // Check used size and mapping count
        assert_eq!(allocator.used(), 6 * PAGE_SIZE);
        assert_eq!(allocator.num_mappings(), 2);
    }

    #[test]
    fn test_va_range_allocator_get_and_remove() {
        let mut allocator = VaRangeAllocator::new(VirtAddr::from(0x1000_0000usize), 0x1000_0000);

        let va = allocator.alloc(2).unwrap();
        assert_eq!(allocator.num_mappings(), 1);

        let retrieved = allocator.get(va);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().num_pages, 2);

        // Interior address lookup must return None (exact start VA required)
        let interior_va = va + VirtAddr::from(PAGE_SIZE);
        assert!(allocator.get(interior_va).is_none());
        assert!(allocator.remove(interior_va).is_none());

        // Removing by exact start VA succeeds
        let removed = allocator.remove(va);
        assert!(removed.is_some());
        assert_eq!(allocator.num_mappings(), 0);
        assert_eq!(allocator.used(), 0);
    }

    #[test]
    fn test_va_range_allocator_first_fit_gap_reuse() {
        let mut allocator = VaRangeAllocator::new(VirtAddr::from(0x1000_0000usize), 10 * PAGE_SIZE);

        // Allocate three regions: [0..4), [4..6), [6..8)
        let va1 = allocator.alloc(4).unwrap();
        let va2 = allocator.alloc(2).unwrap();
        let va3 = allocator.alloc(2).unwrap();
        assert_eq!(va1, VirtAddr::from(0x1000_0000usize));
        assert_eq!(va2, VirtAddr::from(0x1000_4000usize));
        assert_eq!(va3, VirtAddr::from(0x1000_6000usize));
        assert_eq!(allocator.used(), 8 * PAGE_SIZE);

        // Remove the first region [0..4), leaving a 4-page gap at the start
        assert!(allocator.remove(va1).is_some());
        assert_eq!(allocator.used(), 4 * PAGE_SIZE);

        // A 2-page allocation should immediately reuse the start of the gap [0..2)
        let va4 = allocator.alloc(2).unwrap();
        assert_eq!(va4, VirtAddr::from(0x1000_0000usize));
        assert_eq!(allocator.used(), 6 * PAGE_SIZE);

        // Another 2-page allocation should fit in the remainder of the gap [2..4)
        let va5 = allocator.alloc(2).unwrap();
        assert_eq!(va5, VirtAddr::from(0x1000_2000usize));
        assert_eq!(allocator.used(), 8 * PAGE_SIZE);

        // A 3-page allocation cannot fit in the 2-page tail [8..10)
        assert_eq!(allocator.alloc(3), None);

        // A 2-page allocation fits in the tail [8..10)
        let va6 = allocator.alloc(2).unwrap();
        assert_eq!(va6, VirtAddr::from(0x1000_8000usize));
        assert_eq!(allocator.used(), 10 * PAGE_SIZE);
    }

    #[test]
    fn test_global_multi_granule_mapping_helpers() {
        let va = alloc_multi_granule_mapping(2).unwrap();

        // Interior VA must not match
        assert!(free_multi_granule_mapping(va + VirtAddr::from(PAGE_SIZE)).is_none());

        let freed = free_multi_granule_mapping(va);
        assert_eq!(freed.unwrap().num_pages, 2);
        assert!(free_multi_granule_mapping(va).is_none());
    }
}
