use super::address::{align_down, GuestPhysAddr};
use super::page_table::PageTableLevel;
use super::page_table_entry::pte_type;
use super::translation_granule_4k::{RawGPA, RawPTE};
use crate::config::{HUGE_PAGE_SIZE, LARGE_PAGE_SIZE, PAGE_SIZE};
use crate::helper::bits_in_reg;
use core::marker::PhantomData;

/// A generic interface to support all possible page sizes.
//
/// This is defined as a subtrait of Copy to enable #[derive(Clone, Copy)] for Page.
/// Currently, deriving implementations for these traits only works if all dependent types implement it as well.
pub trait PageSize: Copy {
    /// The page size in bytes.
    const SIZE: usize;

    /// The page table level at which a page of this size is mapped
    const MAP_TABLE_LEVEL: usize;

    /// Any extra flag that needs to be set to map a page of this size.
    const MAP_EXTRA_FLAG: u64;
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
    /// Return the stored virtual address.
    pub fn address(&self) -> GuestPhysAddr {
        self.gpa
    }

    /// Flushes this page from the TLB of this CPU.
    pub fn flush_from_tlb(&self) {
        unimplemented!()
    }

    /// Returns a Page including the given virtual address.
    /// That means, the address is rounded down to a page size boundary.
    pub fn including_address(gpa: GuestPhysAddr) -> Self {
        Self {
            gpa: align_down(gpa.as_usize(), S::SIZE).into(),
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

    pub fn table_index<L: PageTableLevel>(&self) -> usize {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        match L::THIS_LEVEL {
            0 => RawGPA::from(self.gpa).get_masked_value(RawGPA::L0Index) as usize,
            1 => RawGPA::from(self.gpa).get_masked_value(RawGPA::L1Index) as usize,
            2 => RawGPA::from(self.gpa).get_masked_value(RawGPA::L2Index) as usize,
            3 => RawGPA::from(self.gpa).get_masked_value(RawGPA::L3Index) as usize,
            _ => panic!(),
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
            self.current.gpa += S::SIZE.into();
            Some(p)
        } else {
            None
        }
    }
}

#[inline]
pub fn get_page_range<S: PageSize>(gpa: GuestPhysAddr, count: usize) -> PageIter<S> {
    let first_page = Page::<S>::including_address(gpa);
    let last_page = Page::<S>::including_address(gpa + ((count - 1) * S::SIZE).into());
    Page::range(first_page, last_page)
}

#[derive(Clone, Copy)]
/// A 4 KiB page mapped in the L3Table.
pub enum BasePageSize {}
impl PageSize for BasePageSize {
    const SIZE: usize = PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 3;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(RawPTE::TYPE, pte_type::TABLE_OR_PAGE);
}

#[derive(Clone, Copy)]
/// A 2 MiB page mapped in the L2Table.
pub enum LargePageSize {}
impl PageSize for LargePageSize {
    const SIZE: usize = LARGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 2;
    const MAP_EXTRA_FLAG: u64 = 0;
}

#[derive(Clone, Copy)]
/// A 1 GiB page mapped in the L1Table.
pub enum HugePageSize {}
impl PageSize for HugePageSize {
    const SIZE: usize = HUGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 1;
    const MAP_EXTRA_FLAG: u64 = 0;
}
