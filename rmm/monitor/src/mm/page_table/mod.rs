use super::address::PhysAddr;
use super::error::Error;
use super::page::{Address, Page, PageIter, PageSize};

use core::marker::PhantomData;

extern crate alloc;

// Safety/TODO:
//  - As of now, concurrency safety for RTT and Realm page table is achieved by a big lock.
//  - If we want to use entry-level locking for a better efficiency, several pieces of codes in this file should be modified accordingly.

pub trait Level {
    const THIS_LEVEL: usize;
    const TABLE_SIZE: usize;
    const TABLE_ALIGN: usize;
    const NUM_ENTRIES: usize;
}

pub trait HasSubtable: Level {
    type NextLevel;
}

pub trait Entry {
    fn new() -> Self;
    fn is_valid(&self) -> bool;
    fn clear(&mut self);

    fn address(&self, level: usize) -> Option<PhysAddr>;

    fn set(&mut self, addr: PhysAddr, flags: u64) -> Result<(), Error>;
    fn set_with_page_table_flags(&mut self, addr: PhysAddr) -> Result<(), Error>;

    // do memory allocation using closure,
    // this is useful if a page table wants entry-level locking,
    // as validity_check and set_* function must be done under entry-level locking.
    // Safety: this default implementation doesn't guarantee entry-level locking.
    //         there is a race window between `is_valid()` and `set_with_page_table_flags()`.
    //         this is only safe if with a big lock. If you want entry-level locking, do override this function properly.
    fn set_with_page_table_flags_via_alloc<T: FnMut() -> usize>(
        &mut self,
        mut alloc: T,
    ) -> Result<(), Error> {
        if !self.is_valid() {
            let table = alloc();
            if table == 0 {
                Err(Error::MmAllocFail)
            } else {
                self.set_with_page_table_flags(PhysAddr::from(table))
            }
        } else {
            Ok(())
        }
    }
    fn index<L: Level>(addr: usize) -> usize;

    fn points_to_table_or_page(&self) -> bool;
}

pub struct PageTable<A, L, E, const N: usize> {
    entries: [E; N],
    level: PhantomData<L>,
    address: PhantomData<A>,
}

pub trait PageTableMethods<A: Address, L, E, const N: usize> {
    fn new(size: usize) -> Result<*mut PageTable<A, L, E, N>, Error>;
    fn new_with_align(size: usize, align: usize) -> Result<*mut PageTable<A, L, E, N>, Error>;
    fn set_pages<S: PageSize>(
        &mut self,
        guest: PageIter<S, A>,
        phys: PageIter<S, PhysAddr>,
        flags: u64,
    ) -> Result<(), Error>;
    fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
    ) -> Result<(), Error>;
    fn entry<S: PageSize, F: FnMut(&mut E)>(
        &mut self,
        guest: Page<S, A>,
        func: F,
    ) -> Result<(), Error>;
    fn drop(&mut self);
    fn unset_page<S: PageSize>(&mut self, guest: Page<S, A>);
}

impl<A: Address, L: Level, E: Entry, const N: usize> PageTableMethods<A, L, E, N>
    for PageTable<A, L, E, N>
{
    fn new(size: usize) -> Result<*mut PageTable<A, L, E, N>, Error> {
        Self::new_with_align(size, 1)
    }

    fn new_with_align(size: usize, align: usize) -> Result<*mut PageTable<A, L, E, N>, Error> {
        let table = unsafe {
            alloc::alloc::alloc_zeroed(
                alloc::alloc::Layout::from_size_align(L::TABLE_SIZE * size, L::TABLE_ALIGN * align)
                    .unwrap(),
            )
        };
        if table as usize == 0 {
            return Err(Error::MmAllocFail);
        }

        let table = table as *mut PageTable<A, L, E, N>;
        unsafe {
            let arr: [E; N] = core::array::from_fn(|_| E::new());
            (*table).entries = arr;
        }
        Ok(table)
    }

    fn set_pages<S: PageSize>(
        &mut self,
        guest: PageIter<S, A>,
        phys: PageIter<S, PhysAddr>,
        flags: u64,
    ) -> Result<(), Error> {
        let mut phys = phys;
        for guest in guest {
            let phys = phys.next().unwrap();
            match self.set_page(guest, phys, flags) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    default fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(guest.address().into());

        // Map page in this level page table
        self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG)
    }

    default fn entry<S: PageSize, F: FnMut(&mut E)>(
        &mut self,
        guest: Page<S, A>,
        mut func: F,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(guest.address().into());
        match self.entries[index].is_valid() {
            true => {
                func(&mut self.entries[index]);
                Ok(())
            }
            false => Err(Error::MmNoEntry),
        }
    }

    default fn drop(&mut self) {
        unsafe {
            alloc::alloc::dealloc(
                self as *mut PageTable<A, L, E, N> as *mut u8,
                alloc::alloc::Layout::from_size_align(L::TABLE_SIZE, L::TABLE_ALIGN).unwrap(),
            );
        }
    }

    default fn unset_page<S: PageSize>(&mut self, guest: Page<S, A>) {
        let index = E::index::<L>(guest.address().into());
        if self.entries[index].is_valid() {
            let res = self.entry(guest, |e| {
                e.clear();
            });

            match res {
                Ok(_) => {
                    info!(
                        "default [L:{}][{}]0x{:X}->cleared",
                        L::THIS_LEVEL,
                        index,
                        guest.address().into()
                    );
                }
                Err(_) => {
                    warn!("unset_page fail");
                }
            }
        }
    }
}

/// This overrides default PageTableMethods for PageTables with subtable.
/// (L0Table, L1Table, L2Table)
/// PageTableMethods for L3 Table remains unmodified.
impl<A: Address, L: HasSubtable, E: Entry /* + Copy*/, const N: usize> PageTableMethods<A, L, E, N>
    for PageTable<A, L, E, N>
where
    L::NextLevel: Level,
{
    fn entry<S: PageSize, F: FnMut(&mut E)>(
        &mut self,
        page: Page<S, A>,
        mut func: F,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        let index = E::index::<L>(page.address().into());

        match self.entries[index].is_valid() {
            true => {
                if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
                    // Need to go deeper (recursive)
                    let subtable = self.subtable::<S>(page);
                    subtable.entry(page, func)
                } else {
                    // The page is either LargePage or HugePage
                    func(&mut self.entries[index]);
                    Ok(())
                }
            }
            false => Err(Error::MmNoEntry),
        }
    }

    fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(guest.address().into());

        if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
            self.entries[index].set_with_page_table_flags_via_alloc(|| {
                let subtable = unsafe {
                    alloc::alloc::alloc_zeroed(
                        alloc::alloc::Layout::from_size_align(
                            L::NextLevel::TABLE_SIZE,
                            L::NextLevel::TABLE_ALIGN,
                        )
                        .unwrap(),
                    )
                } as *mut PageTable<A, L::NextLevel, E, N>;

                if subtable as usize != 0 {
                    let subtable_ptr = subtable as *mut PageTable<A, L::NextLevel, E, N>;
                    unsafe {
                        let arr: [E; N] = core::array::from_fn(|_| E::new());
                        (*subtable_ptr).entries = arr;
                    }
                }

                subtable as usize
            })?;

            // map the page in the subtable (recursive)
            let subtable = self.subtable(guest);
            subtable.set_page(guest, phys, flags)
        } else if L::THIS_LEVEL == S::MAP_TABLE_LEVEL {
            // Map page in this level page table
            self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG)
        } else {
            Err(Error::MmInvalidLevel)
        }
    }

    fn drop(&mut self) {
        for entry in self.entries.iter() {
            //if L::THIS_LEVEL < S::MAP_TABLE_LEVEL && entry.points_to_table_or_page() {
            // if a table which can have subtables points to a table or a page, it should be a table.
            if entry.points_to_table_or_page() {
                let subtable_addr = entry.address(L::THIS_LEVEL).unwrap();
                let subtable: &mut PageTable<A, L::NextLevel, E, N> = unsafe {
                    &mut *(subtable_addr.as_usize() as *mut PageTable<A, L::NextLevel, E, N>)
                };
                subtable.drop();
            }
        }
        unsafe {
            alloc::alloc::dealloc(
                self as *mut PageTable<A, L, E, N> as *mut u8,
                alloc::alloc::Layout::from_size_align(L::TABLE_SIZE, L::TABLE_SIZE).unwrap(),
            );
        }
    }

    fn unset_page<S: PageSize>(&mut self, guest: Page<S, A>) {
        let index = E::index::<L>(guest.address().into());

        if self.entries[index].is_valid() {
            let res = self.entry(guest, |e| {
                e.clear();
            });
            if res.is_err() {
                warn!("unset_page fail");
            }
        }
    }
}

impl<A: Address, L: HasSubtable, E: Entry, const N: usize> PageTable<A, L, E, N>
where
    L::NextLevel: Level,
{
    /// Returns the next subtable for the given page in the page table hierarchy.
    fn subtable<S: PageSize>(&self, page: Page<S, A>) -> &mut PageTable<A, L::NextLevel, E, N> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(page.address().into());
        let subtable_addr = self.entries[index].address(L::THIS_LEVEL).unwrap();
        unsafe { &mut *(subtable_addr.as_usize() as *mut PageTable<A, L::NextLevel, E, N>) }
    }
}
