use super::address::{to_paddr, to_vaddr};
use super::address::{GuestPhysAddr, PhysAddr};
use super::page_table_entry::{Page, PageIter, PageSize, PageTableEntry, PageTableEntryFlags};
use super::pgtlb_allocator;
use crate::config::PAGE_SIZE;
use core::marker::PhantomData;
use core::mem;
use monitor::error::Error;

/// An interface to allow for a generic implementation of struct PageTable
/// for all 4 levels.
/// Must be implemented by all page tables.
pub trait PageTableLevel {
    /// Numeric page table level
    const THIS_LEVEL: usize;
}

/// PageTableLevels
pub enum L0Table {}
pub enum L1Table {}
pub enum L2Table {}
pub enum L3Table {}

/// An interface for page tables with sub page tables
/// (all except L3Table).
/// Having both PageTableLevel and LevelHierarchy leverages Rust's typing system
///  to provide a subtable method only for those that have sub page tables.
pub trait LevelHierarchy: PageTableLevel {
    type NextLevel;
}

pub trait TranslationGranule {
    fn table_index<S: PageSize>(&self, page: Page<S>) -> usize;
    fn table_addr_mask(&self) -> usize;
    fn page_addr_mask(&self) -> usize;
}

/// Representation of any page table in memory.
/// Parameter L supplies information for Rust's typing system
/// to distinguish between the different tables.
pub struct PageTable<L> {
    /// Each page table has 512 entries (can be calculated using PAGE_MAP_BITS).
    entries: [PageTableEntry; (PAGE_SIZE / mem::size_of::<usize>())],

    /// Required by Rust to support the L parameter.
    level: PhantomData<L>,
}

/// A trait defining methods every page table has to implement.
/// This additional trait is necessary to make use of Rust's specialization
///  feature and provide a default implementation of some methods.
trait PageTablePrivateMethods {
    fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry>;
    fn map_page_in_this_table<S: PageSize>(
        &mut self,
        page: Page<S>,
        paddr: PhysAddr,
        flags: PageTableEntryFlags,
    ) -> Result<(), Error>;
    fn map_page<S: PageSize>(
        &mut self,
        page: Page<S>,
        paddr: PhysAddr,
        flags: PageTableEntryFlags,
    ) -> Result<(), Error>;
}

impl<L: PageTableLevel> PageTablePrivateMethods for PageTable<L> {
    /// Returns the PageTableEntry for the given page if it is present,
    /// otherwise returns None.
    ///
    /// This is the default implementation called only for L3Table.
    /// It is overridden by a specialized implementation
    /// for all tables with sub tables (all except L3Table).
    default fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry> {
        assert_eq!(L::THIS_LEVEL, S::MAP_TABLE_LEVEL);
        let index = self.table_index::<S>(page);

        if self.entries[index].is_valid() {
            Some(self.entries[index])
        } else {
            None
        }
    }

    /// Maps a single page in this table to the given physical address.
    ///
    /// Must only be called if a page of this size
    /// is mapped at this page table level!
    fn map_page_in_this_table<S: PageSize>(
        &mut self,
        page: Page<S>,
        paddr: PhysAddr,
        flags: PageTableEntryFlags,
    ) -> Result<(), Error> {
        assert_eq!(L::THIS_LEVEL, S::MAP_TABLE_LEVEL);
        assert_eq!(paddr & !page.mask(), 0);
        let index = self.table_index::<S>(page);
        let flush = self.entries[index].is_valid();

        if flags == PageTableEntryFlags::BLANK {
            // in this case we unmap the pages
            self.entries[index].set_pte(paddr, flags, self.page_addr_mask());
        } else {
            self.entries[index].set_pte(paddr, S::MAP_EXTRA_FLAG | flags, self.page_addr_mask());
        }

        if flush {
            page.flush_from_tlb();
        }
        Ok(())
    }

    /// Maps a single page to the given physical address.
    //
    /// This is the default implementation that just calls
    /// the map_page_in_this_table method. It is overridden by a specialized
    /// implementation for all tables with sub tables (all except L3Table).
    default fn map_page<S: PageSize>(
        &mut self,
        page: Page<S>,
        paddr: PhysAddr,
        flags: PageTableEntryFlags,
    ) -> Result<(), Error> {
        self.map_page_in_this_table::<S>(page, paddr, flags)
    }
}

impl<L: LevelHierarchy> PageTablePrivateMethods for PageTable<L>
where
    L::NextLevel: PageTableLevel,
{
    /// Returns the PageTableEntry for the given page if it is present,
    /// otherwise returns None.
    //
    /// This is the implementation for all tables with subtables (L0Table,
    /// L1Table, L2Table). It overrides the default implementation above.
    fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        let index = self.table_index::<S>(page);

        if self.entries[index].is_valid() {
            if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
                let subtable = self.subtable::<S>(page);
                subtable.get_page_table_entry::<S>(page)
            } else {
                Some(self.entries[index])
            }
        } else {
            None
        }
    }

    /// Maps a single page to the given physical address.
    //
    /// This is the implementation for all tables with subtables
    /// (L0Table, L1Table, L2Table). It overrides the default implementation
    fn map_page<S: PageSize>(
        &mut self,
        page: Page<S>,
        paddr: PhysAddr,
        flags: PageTableEntryFlags,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);

        if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
            let index = self.table_index::<S>(page);

            // Does the table exist yet?
            if !self.entries[index].is_valid() {
                // Allocate a single page for the new entry
                // and mark it as a valid, writable subtable.
                let subtable: *mut PageTable<L::NextLevel> =
                    pgtlb_allocator::allocate_tables(1, PAGE_SIZE).unwrap();

                let subtable_paddr = to_paddr(subtable as usize);
                self.entries[index].set_pte(
                    subtable_paddr,
                    PageTableEntryFlags::VALID | PageTableEntryFlags::TABLE_OR_PAGE_DESC,
                    self.table_addr_mask(),
                );

                // Mark all entries as unused in the newly created table.
                let subtable = self.subtable::<S>(page);
                for entry in subtable.entries.iter_mut() {
                    entry.set_pte(PhysAddr::zero(), PageTableEntryFlags::BLANK, PAGE_SIZE);
                }
            }

            let subtable = self.subtable::<S>(page);
            subtable.map_page::<S>(page, paddr, flags)?;
        } else {
            // Calling the default implementation from a specialized one
            // is not supported (yet), so we have to resort to an extra function.
            self.map_page_in_this_table::<S>(page, paddr, flags)?;
        }
        Ok(())
    }
}

impl<L: LevelHierarchy> PageTable<L>
where
    L::NextLevel: PageTableLevel,
{
    /// Returns the next subtable for the given page in the page table hierarchy.
    //
    /// Must only be called if a page of this size is mapped in a subtable!
    fn subtable<S: PageSize>(&self, page: Page<S>) -> &mut PageTable<L::NextLevel> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        // Calculate the address of the subtable.
        let index = self.table_index::<S>(page);
        let subtable_paddr = self.entries[index].output_address(self.table_addr_mask());
        let subtable_address = to_vaddr(subtable_paddr);
        unsafe { &mut *(subtable_address as *mut PageTable<L::NextLevel>) }
    }

    /// Maps a continuous range of pages.
    //
    /// # Arguments
    //
    /// * `range` - The range of pages of size S
    /// * `paddr` - First physical address to map these pages to
    /// * `flags` - Flags from PageTableEntryFlags to set for the page table
    ///             entry (e.g. WRITABLE or EXECUTE_DISABLE).
    ///             The PRESENT and ACCESSED are already set automatically.
    pub fn map_pages<S: PageSize>(
        &mut self,
        range: PageIter<S>,
        paddr: PhysAddr,
        flags: PageTableEntryFlags,
    ) -> Result<(), Error> {
        let mut current_paddr = paddr;

        for page in range {
            self.map_page::<S>(page, current_paddr, flags)?;
            current_paddr += S::SIZE as usize;
        }
        Ok(())
    }
}

#[inline]
pub fn get_page_range<S: PageSize>(gpa: GuestPhysAddr, count: usize) -> PageIter<S> {
    let first_page = Page::<S>::including_address(gpa);
    let last_page = Page::<S>::including_address(gpa + (count - 1) * S::SIZE);
    Page::range(first_page, last_page)
}
