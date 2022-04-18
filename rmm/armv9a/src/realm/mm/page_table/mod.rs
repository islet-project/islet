use monitor::mm::page_table::{self, Entry as PTEntry, HasSubtable};
use monitor::realm::mm::address::PhysAddr;

use super::page::{Page, PageIter, PageSize};
use super::translation_granule_4k::{RawPTE, PAGE_MAP_BITS};
use crate::config::PAGE_SIZE;
use crate::helper::bits_in_reg;
use core::marker::PhantomData;
use entry::Entry;
use monitor::const_assert_size;

mod allocator;
pub mod entry;
pub mod pte;

pub struct PageTable<L, E> {
    entries: [E; 1 << PAGE_MAP_BITS],
    level: PhantomData<L>,
}

/// The Level 0 Table
pub enum L0Table {}
impl page_table::Level for L0Table {
    const THIS_LEVEL: usize = 0;
}
impl HasSubtable for L0Table {
    type NextLevel = L1Table;
}

/// The Level 1 Table
pub enum L1Table {}
impl page_table::Level for L1Table {
    const THIS_LEVEL: usize = 1;
}
impl HasSubtable for L1Table {
    type NextLevel = L2Table;
}

/// The Level 2 Table
pub enum L2Table {}
impl page_table::Level for L2Table {
    const THIS_LEVEL: usize = 2;
}
impl HasSubtable for L2Table {
    type NextLevel = L3Table;
}

/// The Level 3 Table (Doesn't have Subtable!)
pub enum L3Table {}
impl page_table::Level for L3Table {
    const THIS_LEVEL: usize = 3;
}

const_assert_size!(PageTable<L0Table, Entry>, PAGE_SIZE);

pub trait PageTableMethods<L, E> {
    fn new(size: usize) -> Result<*mut PageTable<L, E>, ()>;
    fn map_multiple_pages<S: PageSize>(&mut self, range: PageIter<S>, paddr: PhysAddr, flags: u64);

    // will be specialized
    fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<E>;
    fn map_page<S: PageSize>(&mut self, page: Page<S>, paddr: PhysAddr, flags: u64);
}

impl<L: page_table::Level, E: PTEntry + Copy> PageTableMethods<L, E> for PageTable<L, E> {
    fn new(size: usize) -> Result<*mut PageTable<L, E>, ()> {
        let table = allocator::alloc(size)?;

        unsafe {
            (*table).entries = [E::new(); 1 << PAGE_MAP_BITS];
        }

        Ok(table)
    }

    /// Maps a continuous range of pages.
    ///
    /// # Arguments
    ///
    /// * `range` - The range of pages of size S
    /// * `paddr` - First physical address to map these pages to
    /// * `flags` - Flags to set for the page table entry (e.g. WRITABLE or EXECUTE_DISABLE).
    ///             The VALID and AF will be set automatically.
    fn map_multiple_pages<S: PageSize>(&mut self, range: PageIter<S>, paddr: PhysAddr, flags: u64) {
        let mut current_paddr = paddr;

        for page in range {
            self.map_page::<S>(page, current_paddr, flags);
            current_paddr += S::SIZE.into();
        }
    }

    /// Returns the Entry for the given page if it is valid,
    /// otherwise returns None.
    ///
    /// This is the default implementation called only for L3Table.
    /// It is overridden by a specialized implementation for all tables with subtables.
    default fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<E> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = page.table_index::<L>();
        match self.entries[index].is_valid() {
            true => Some(self.entries[index]),
            false => None,
        }
    }

    /// Maps a single page to the given physical address.
    //
    /// This is the default implementation called only for L3Table.
    /// It is overridden by a specialized implementation for all tables with sub tables.
    default fn map_page<S: PageSize>(&mut self, page: Page<S>, paddr: PhysAddr, flags: u64) {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = page.table_index::<L>();

        // Map page in this level page table
        self.entries[index].set(paddr, flags | S::MAP_EXTRA_FLAG);
    }
}

/// This overrides default PageTableMethods for PageTables with subtable.
/// (L0Table, L1Table, L2Table)
/// PageTableMethods for L3 Table remains unmodified.
impl<L: HasSubtable, E: PTEntry + Copy> PageTableMethods<L, E> for PageTable<L, E>
where
    L::NextLevel: page_table::Level,
{
    fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<E> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        let index = page.table_index::<L>();

        match self.entries[index].is_valid() {
            true => {
                if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
                    // Need to go deeper (recursive)
                    let subtable = self.subtable::<S>(page);
                    subtable.get_page_table_entry::<S>(page)
                } else {
                    // The page is either LargePage or HugePage
                    Some(self.entries[index])
                }
            }
            false => None,
        }
    }

    fn map_page<S: PageSize>(&mut self, page: Page<S>, paddr: PhysAddr, flags: u64) {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);

        let index = page.table_index::<L>();

        if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
            // Need to go deeper (recursive)
            if !self.entries[index].is_valid() {
                // The subtable is not yet there. Let's create one

                let subtable = PageTable::<L::NextLevel, Entry>::new(1).unwrap();
                let subtable_paddr = PhysAddr::from(subtable);

                self.entries[index].set(
                    subtable_paddr,
                    bits_in_reg(RawPTE::ATTR, pte::attribute::NORMAL)
                        | bits_in_reg(RawPTE::TYPE, pte::page_type::TABLE_OR_PAGE),
                );
            }

            // map the page in the subtable (recursive)
            let subtable = self.subtable::<S>(page);
            subtable.map_page::<S>(page, paddr, flags);
        } else if L::THIS_LEVEL == S::MAP_TABLE_LEVEL {
            // Map page in this level page table
            self.entries[index].set(paddr, flags | S::MAP_EXTRA_FLAG);
        }
    }
}

impl<L: HasSubtable, E: PTEntry> PageTable<L, E>
where
    L::NextLevel: page_table::Level,
{
    /// Returns the next subtable for the given page in the page table hierarchy.
    fn subtable<S: PageSize>(&self, page: Page<S>) -> &mut PageTable<L::NextLevel, E> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        let index = page.table_index::<L>();
        let subtable_addr = self.entries[index].address(L::THIS_LEVEL).unwrap();
        unsafe { &mut *(subtable_addr.as_usize() as *mut PageTable<L::NextLevel, E>) }
    }
}
