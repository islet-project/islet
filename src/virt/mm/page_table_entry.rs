use super::address::{GuestPhysAddr, PhysAddr};
use crate::define_mask;
use core::marker::PhantomData;

bitflags! {
    /// Useful flags for an entry in either table (L0Table, L1Table, L2Table, L3Table).
    //
    /// See ARM Architecture Reference Manual, ARMv8, for ARMv8-A Reference Profile, Issue C.a, Chapter D4.3.3
    pub struct PageTableEntryFlags: usize {
        /// Execute Never
        const XN = 1 << 54;

        /// Continuous bit for TLB caching
        const CONT = 1 << 52;

        /// Access flag
        const AF = 1 << 10;

        /// Shareability
        const SH_INNER = 1 << 9 | 1 << 8;

        /// Stage 2 data Access Permissions
        const S2AP_RW = 1 << 7 | 1 << 6;
        const S2AP_WO = 1 << 7 | 0 << 6;
        const S2AP_RO = 0 << 7 | 1 << 6;
        const S2AP_NONE = 0 << 7 | 0 << 6;

        /// Stage 2 memory attributes
        const MEMATTR_NORMAL = 1 << 4 | 0 << 3 | 0 << 2;
        const MEMATTR_NORMAL_NC = 0 << 4 | 1 << 3 | 1 << 2; // non-cacheable
        // NG: non-gathering, NR: non-reordering, E: early write ack
        //const MEMATTR_DEVICE_NGNRE = 0 << 4 | 0 << 3 | 1 << 2;
        const MEMATTR_DEVICE_NGNRE = 0 << 4 | 0 << 3 | 1 << 2;

        /// Set if this entry points to a table or a block/page.
        const TABLE_OR_PAGE_DESC = 1 << 1;

        /// Set if this entry is valid.
        const VALID = 1 << 0;
    }
}

impl PageTableEntryFlags {
    //// An empty set of flags for unused/zeroed table entries.
    //// Needed as long as empty() is no const function.
    pub const BLANK: PageTableEntryFlags = PageTableEntryFlags { bits: 0 };
    const FLAG_MASK: usize = define_mask!(63, 52) | define_mask!(11, 0);
}

/// An entry in either table
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    /// Physical memory address this entry refers, combined with flags from PageTableEntryFlags.
    /// contains information: physical address, memory attributes, access permission
    pte: usize,
}

impl PageTableEntry {
    /// Return the stored physical address.
    pub fn output_address(&self, mask: usize) -> PhysAddr {
        PhysAddr(self.pte & mask & !(usize::MAX << 48))
    }

    pub fn flags(&self) -> PageTableEntryFlags {
        PageTableEntryFlags {
            bits: self.pte & PageTableEntryFlags::FLAG_MASK,
        }
    }

    /// Returns whether this entry is valid (present).
    pub fn is_valid(&self) -> bool {
        (self.pte & PageTableEntryFlags::VALID.bits()) != 0
    }

    /// Mark this as a valid (present) entry and set address translation and flags.
    //
    /// # Arguments
    //
    /// * `paddr` - The physical memory address this entry shall translate to
    /// * `flags` - Flags from PageTableEntryFlags (note that the VALID, and ACCESSED flags are set automatically)
    pub fn set_pte(&mut self, paddr: PhysAddr, flags: PageTableEntryFlags, mask: usize) {
        assert_eq!(
            paddr & !mask,
            0,
            "Physical address is not on a {:#X} boundary (paddr = {:#X})",
            mask,
            paddr.as_usize()
        );
        let mut flags_to_set = flags;
        // TODO: validate flags and set additional flags
        flags_to_set.insert(PageTableEntryFlags::SH_INNER);
        flags_to_set.insert(PageTableEntryFlags::AF);
        self.pte = paddr.as_usize() | flags_to_set.bits();
    }

    pub fn get_pte(&self) -> usize {
        self.pte as usize
    }
}

/// A generic interface to support all possible page sizes.
//
/// This is defined as a subtrait of Copy to enable #[derive(Clone, Copy)] for Page.
/// Currently, deriving implementations for these traits only works if all dependent types implement it as well.
pub trait PageSize: Copy {
    /// The page size in bytes.
    /// const SIZE: usize;
    const SIZE: usize;

    /// The page table level at which a page of this size is mapped
    const MAP_TABLE_LEVEL: usize;

    /// Any extra flag that needs to be set to map a page of this size.
    /// For example: PageTableEntryFlags::TABLE_OR_4KIB_PAGE.
    const MAP_EXTRA_FLAG: PageTableEntryFlags;
}

/// A memory page of the size given by S.
#[derive(Clone, Copy)]
pub struct Page<S: PageSize> {
    /// Virtual memory address of this page.
    /// This is rounded to a page size boundary on creation.
    gpa: GuestPhysAddr,

    /// Required by Rust to support the S parameter.
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    pub fn mask(&self) -> usize {
        (1 << 48) - S::SIZE
    }

    /// Return the stored virtual address.
    pub fn address(&self) -> GuestPhysAddr {
        self.gpa
    }

    /// Flushes this page from the TLB of this CPU.
    // FIXME: covert to gpa
    pub fn flush_from_tlb(&self) {
        // TODO:
    }

    /// Returns whether the given virtual address is a valid one in the AArch64 memory model.
    //
    /// Current AArch64 supports only 48-bit for virtual memory addresses.
    /// The upper bits must always be 0 or 1 and indicate whether TBBR0 or TBBR1 contains the
    /// base address. So always enforce 0 here.
    fn is_valid_address(gpa: GuestPhysAddr) -> bool {
        gpa < GuestPhysAddr(0x1_0000_0000_0000)
    }

    /// Returns a Page including the given virtual address.
    /// That means, the address is rounded down to a page size boundary.
    pub fn including_address(gpa: GuestPhysAddr) -> Self {
        assert!(
            Self::is_valid_address(gpa),
            "Guest Physical address {:#X} is invalid",
            gpa.as_usize()
        );

        Self {
            gpa: gpa.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns a PageIter to iterate from the given first Page to the given last Page (inclusive).
    pub fn range(first: Self, last: Self) -> PageIter<S> {
        assert!(first.gpa <= last.gpa);
        PageIter {
            current: first,
            last: last,
        }
    }
}

/// An iterator to walk through a range of pages of size S.
pub struct PageIter<S: PageSize> {
    current: Page<S>,
    last: Page<S>,
}

impl<S: PageSize> Iterator for PageIter<S> {
    type Item = Page<S>;

    fn next(&mut self) -> Option<Page<S>> {
        if self.current.gpa <= self.last.gpa {
            let p = self.current;
            self.current.gpa += S::SIZE;
            Some(p)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub enum BasePageSize {}

#[derive(Clone, Copy)]
pub enum LargePageSize {}

#[derive(Clone, Copy)]
pub enum HugePageSize {}
