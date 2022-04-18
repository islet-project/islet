use monitor::const_assert_size;
use monitor::mm::address::PhysAddr;
use monitor::mm::page::{Page, PageIter, PageSize};
use monitor::mm::page_table::{self, Entry as PTEntry, HasSubtable};
use monitor::realm::mm::address::GuestPhysAddr;

use super::page::table_index;
use super::translation_granule_4k::{RawPTE, PAGE_MAP_BITS};
use crate::config::PAGE_SIZE;
use crate::helper::bits_in_reg;
use entry::Entry;

use core::marker::PhantomData;
use core::ops::{Add, AddAssign};

mod allocator;
pub mod entry;
pub mod pte;

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

const_assert_size!(PageTable<GuestPhysAddr, L0Table, Entry>, PAGE_SIZE);

pub struct PageTable<A, L, E> {
    entries: [E; 1 << PAGE_MAP_BITS],
    level: PhantomData<L>,
    address: PhantomData<A>,
}

pub trait PageTableMethods<A: Add + AddAssign + Copy + From<usize> + Into<usize> + PartialOrd, L, E>
{
    fn new(size: usize) -> Result<*mut PageTable<A, L, E>, ()>;
    fn set_pages<S: PageSize>(
        &mut self,
        guest: PageIter<S, A>,
        phys: PageIter<S, PhysAddr>,
        flags: u64,
    );
    fn set_page<S: PageSize>(&mut self, guest: Page<S, A>, phys: Page<S, PhysAddr>, flags: u64);

    fn entry<S: PageSize>(&self, guest: Page<S, A>) -> Option<E>;
}

impl<
        A: Add + AddAssign + Copy + From<usize> + Into<usize> + PartialOrd,
        L: page_table::Level,
        E: PTEntry + Copy,
    > PageTableMethods<A, L, E> for PageTable<A, L, E>
{
    fn new(size: usize) -> Result<*mut PageTable<A, L, E>, ()> {
        let table = allocator::alloc(size)?;

        unsafe {
            (*table).entries = [E::new(); 1 << PAGE_MAP_BITS];
        }

        Ok(table)
    }

    fn set_pages<S: PageSize>(
        &mut self,
        guest: PageIter<S, A>,
        phys: PageIter<S, PhysAddr>,
        flags: u64,
    ) {
        let mut phys = phys;
        for guest in guest {
            let phys = phys.next().unwrap();
            self.set_page::<S>(guest, phys, flags);
        }
    }

    default fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
    ) {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = table_index::<S, A, L>(guest);

        // Map page in this level page table
        self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG);
    }

    default fn entry<S: PageSize>(&self, guest: Page<S, A>) -> Option<E> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = table_index::<S, A, L>(guest);
        match self.entries[index].is_valid() {
            true => Some(self.entries[index]),
            false => None,
        }
    }
}

/// This overrides default PageTableMethods for PageTables with subtable.
/// (L0Table, L1Table, L2Table)
/// PageTableMethods for L3 Table remains unmodified.
impl<
        A: Add + AddAssign + Copy + From<usize> + Into<usize> + PartialOrd,
        L: HasSubtable,
        E: PTEntry + Copy,
    > PageTableMethods<A, L, E> for PageTable<A, L, E>
where
    L::NextLevel: page_table::Level,
{
    fn entry<S: PageSize>(&self, page: Page<S, A>) -> Option<E> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        let index = table_index::<S, A, L>(page);

        match self.entries[index].is_valid() {
            true => {
                if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
                    // Need to go deeper (recursive)
                    let subtable = self.subtable::<S>(page);
                    subtable.entry::<S>(page)
                } else {
                    // The page is either LargePage or HugePage
                    Some(self.entries[index])
                }
            }
            false => None,
        }
    }

    fn set_page<S: PageSize>(&mut self, guest: Page<S, A>, phys: Page<S, PhysAddr>, flags: u64) {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);

        let index = table_index::<S, A, L>(guest);

        if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
            // Need to go deeper (recursive)
            if !self.entries[index].is_valid() {
                // The subtable is not yet there. Let's create one

                let subtable = PageTable::<A, L::NextLevel, E>::new(1).unwrap();
                let subtable_paddr = PhysAddr::from(subtable);

                self.entries[index].set(
                    subtable_paddr,
                    bits_in_reg(RawPTE::ATTR, pte::attribute::NORMAL)
                        | bits_in_reg(RawPTE::TYPE, pte::page_type::TABLE_OR_PAGE),
                );
            }

            // map the page in the subtable (recursive)
            let subtable = self.subtable(guest);
            subtable.set_page::<S>(guest, phys, flags);
        } else if L::THIS_LEVEL == S::MAP_TABLE_LEVEL {
            // Map page in this level page table
            self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG);
        }
    }
}

impl<
        A: Add + AddAssign + Copy + From<usize> + Into<usize> + PartialOrd,
        L: HasSubtable,
        E: PTEntry,
    > PageTable<A, L, E>
where
    L::NextLevel: page_table::Level,
{
    /// Returns the next subtable for the given page in the page table hierarchy.
    fn subtable<S: PageSize>(&self, page: Page<S, A>) -> &mut PageTable<A, L::NextLevel, E> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        let index = table_index::<S, A, L>(page);
        let subtable_addr = self.entries[index].address(L::THIS_LEVEL).unwrap();
        unsafe { &mut *(subtable_addr.as_usize() as *mut PageTable<A, L::NextLevel, E>) }
    }
}
