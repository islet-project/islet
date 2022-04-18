use super::address::PhysAddr;
use super::page::{Address, Page, PageIter, PageSize};

use core::marker::PhantomData;

extern crate alloc;

//TODO remove this
pub const NUM_OF_ENTRIES: usize = 1 << 9;

pub trait Level {
    const THIS_LEVEL: usize;
}

pub trait HasSubtable: Level {
    type NextLevel;
}

pub trait Entry {
    fn new() -> Self;
    fn is_valid(&self) -> bool;
    fn clear(&mut self);

    fn address(&self, level: usize) -> Option<PhysAddr>;

    fn set(&mut self, addr: PhysAddr, flags: u64);
    fn set_with_page_table_flags(&mut self, addr: PhysAddr);

    fn index<L: Level>(addr: usize) -> usize;
}

pub struct PageTable<A, L, E> {
    entries: [E; NUM_OF_ENTRIES],
    level: PhantomData<L>,
    address: PhantomData<A>,
}

pub trait PageTableMethods<A: Address, L, E> {
    fn new<S: PageSize>(size: usize) -> Result<*mut PageTable<A, L, E>, ()>;
    fn new_with_align<S: PageSize>(
        size: usize,
        align: usize,
    ) -> Result<*mut PageTable<A, L, E>, ()>;
    fn set_pages<S: PageSize>(
        &mut self,
        guest: PageIter<S, A>,
        phys: PageIter<S, PhysAddr>,
        flags: u64,
    );
    fn set_page<S: PageSize>(&mut self, guest: Page<S, A>, phys: Page<S, PhysAddr>, flags: u64);
    fn entry<S: PageSize>(&self, guest: Page<S, A>) -> Option<E>;
}

impl<A: Address, L: Level, E: Entry + Copy> PageTableMethods<A, L, E> for PageTable<A, L, E> {
    fn new<S: PageSize>(size: usize) -> Result<*mut PageTable<A, L, E>, ()> {
        Self::new_with_align::<S>(size, 1)
    }

    fn new_with_align<S: PageSize>(
        size: usize,
        align: usize,
    ) -> Result<*mut PageTable<A, L, E>, ()> {
        let table = unsafe {
            alloc::alloc::alloc_zeroed(
                alloc::alloc::Layout::from_size_align(S::SIZE * size, S::SIZE * align).unwrap(),
            )
        };

        assert_ne!(table, 0 as *mut _);

        let table = table as *mut PageTable<A, L, E>;

        unsafe {
            (*table).entries = [E::new(); NUM_OF_ENTRIES];
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
            self.set_page(guest, phys, flags);
        }
    }

    default fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
    ) {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(guest.address().into());

        // Map page in this level page table
        self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG);
    }

    default fn entry<S: PageSize>(&self, guest: Page<S, A>) -> Option<E> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(guest.address().into());
        match self.entries[index].is_valid() {
            true => Some(self.entries[index]),
            false => None,
        }
    }
}

/// This overrides default PageTableMethods for PageTables with subtable.
/// (L0Table, L1Table, L2Table)
/// PageTableMethods for L3 Table remains unmodified.
impl<A: Address, L: HasSubtable, E: Entry + Copy> PageTableMethods<A, L, E> for PageTable<A, L, E>
where
    L::NextLevel: Level,
{
    fn entry<S: PageSize>(&self, page: Page<S, A>) -> Option<E> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        let index = E::index::<L>(page.address().into());

        match self.entries[index].is_valid() {
            true => {
                if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
                    // Need to go deeper (recursive)
                    let subtable = self.subtable::<S>(page);
                    subtable.entry(page)
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

        let index = E::index::<L>(guest.address().into());

        if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
            if !self.entries[index].is_valid() {
                let subtable = unsafe {
                    alloc::alloc::alloc_zeroed(
                        alloc::alloc::Layout::from_size_align(S::TABLE_SIZE, S::TABLE_SIZE)
                            .unwrap(),
                    )
                } as *mut PageTable<A, L, E>;

                self.entries[index].set_with_page_table_flags(PhysAddr::from(subtable));
            }

            // map the page in the subtable (recursive)
            let subtable = self.subtable(guest);
            subtable.set_page(guest, phys, flags);
        } else if L::THIS_LEVEL == S::MAP_TABLE_LEVEL {
            // Map page in this level page table
            self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG);
        }
    }
}

impl<A: Address, L: HasSubtable, E: Entry> PageTable<A, L, E>
where
    L::NextLevel: Level,
{
    /// Returns the next subtable for the given page in the page table hierarchy.
    fn subtable<S: PageSize>(&self, page: Page<S, A>) -> &mut PageTable<A, L::NextLevel, E> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(page.address().into());
        let subtable_addr = self.entries[index].address(L::THIS_LEVEL).unwrap();
        unsafe { &mut *(subtable_addr.as_usize() as *mut PageTable<A, L::NextLevel, E>) }
    }
}
