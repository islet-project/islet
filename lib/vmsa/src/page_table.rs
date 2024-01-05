extern crate alloc;

use super::address::{Address, PhysAddr};
use super::error::Error;
use super::guard::EntryGuard;
use super::page::{Page, PageIter, PageSize};

use alloc::alloc::Layout;
use core::marker::PhantomData;

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
    /// Inner represents a inner type encapsulated in Entry (e.g., Inner=u64 for struct Entry<u64>)
    type Inner;

    fn new() -> Self;
    fn is_valid(&self) -> bool;
    fn clear(&mut self);

    fn pte(&self) -> u64;
    fn mut_pte(&mut self) -> &mut Self::Inner;
    fn address(&self, level: usize) -> Option<PhysAddr>;

    fn set(&mut self, addr: PhysAddr, flags: u64, is_raw: bool) -> Result<(), Error>;
    fn set_with_page_table_flags(&mut self, addr: PhysAddr) -> Result<(), Error>;

    // returns EntryGuard which allows accessing what's inside Entry while holding a proper lock.
    fn lock(&self) -> Result<Option<EntryGuard<'_, Self::Inner>>, Error> {
        Err(Error::MmUnimplemented)
    }

    // do memory allocation using closure,
    // this is useful if a page table wants entry-level locking,
    // as validity_check and set_* function must be done under entry-level locking.
    // Safety: this  implementation doesn't guarantee entry-level locking.
    //         there is a race window between `is_valid()` and `set_with_page_table_flags()`.
    //         this is only safe if with a big lock. If you want entry-level locking, do override this function properly.
    fn set_with_page_table_flags_via_alloc<T: FnMut() -> usize>(
        &mut self,
        _index: usize,
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

    fn subtable(&self, _index: usize, level: usize) -> Result<usize, Error> {
        match self.address(level) {
            Some(addr) => Ok(addr.as_usize()),
            _ => Err(Error::MmInvalidAddr),
        }
    }

    fn points_to_table_or_page(&self) -> bool;
}

/// Safety: the caller must do proper error handling if it's failed to allocate memory
pub trait MemAlloc {
    unsafe fn alloc(layout: Layout) -> *mut u8 {
        alloc::alloc::alloc(layout)
    }

    unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
        alloc::alloc::alloc_zeroed(layout)
    }

    unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
        alloc::alloc::dealloc(ptr, layout);
    }
}

pub struct PageTable<A, L, E, const N: usize> {
    entries: [E; N],
    level: PhantomData<L>,
    address: PhantomData<A>,
}

impl<A: Address, L: Level, E: Entry, const N: usize> MemAlloc for PageTable<A, L, E, N> {}
impl<A: Address, L: HasSubtable, E: Entry, const N: usize> MemAlloc for PageTable<A, L, E, N> {}

pub trait PageTableMethods<A: Address, L, E: Entry, const N: usize> {
    fn new_with_base(base: usize) -> Result<*mut PageTable<A, L, E, N>, Error>;
    fn new_with_align(size: usize, align: usize) -> Result<*mut PageTable<A, L, E, N>, Error>;
    /// Sets multiple page table entries
    ///
    /// (input)
    ///    guest : an iterator of target guest addresses to modify their page table entry mapping
    ///    phys  : an iterator of target physical addresses to be mapped
    ///    flags : flags to attach
    ///    is_raw: (if on) a user-provided `flags` is only attached (== without attaching a default flag)
    fn set_pages<S: PageSize>(
        &mut self,
        guest: PageIter<S, A>,
        phys: PageIter<S, PhysAddr>,
        flags: u64,
        is_raw: bool,
    ) -> Result<(), Error>;
    /// Sets a single page table entry
    ///
    /// (input)
    ///    guest : a target guest address to modify its page table entry mapping
    ///    phys  : a target physical address to be mapped
    ///    flags : flags to attach
    ///    is_raw: (if on) a user-provided `flags` is only attached (== without attaching a default flag)
    fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
        is_raw: bool,
    ) -> Result<(), Error>;
    /// Traverses page table entries recursively and calls the callback for the lastly reached entry
    ///
    /// (input)
    ///    guest: a target guest page to translate
    ///    level: the intended page-table level to reach
    ///    no_valid_check: (if on) omits a validity check which is irrelevant in stage 2 TTE
    ///    func : the callback to be processed
    ///
    /// (output)
    ///    if exists,
    ///      A tuple of
    ///        ((EntryGuard), the lastly reached page-table level (usize))
    ///    else,
    ///      None
    fn entry<S: PageSize, F: FnMut(&mut E) -> Result<Option<EntryGuard<'_, E::Inner>>, Error>>(
        &mut self,
        guest: Page<S, A>,
        level: usize,
        no_valid_check: bool,
        func: F,
    ) -> Result<(Option<EntryGuard<'_, E::Inner>>, usize), Error>;
    fn drop(&mut self);
    fn unset_page<S: PageSize>(&mut self, guest: Page<S, A>);
}

impl<A: Address, L: Level, E: Entry, const N: usize> PageTable<A, L, E, N> {
    pub fn new() -> Self {
        Self {
            entries: core::array::from_fn(|_| E::new()),
            level: PhantomData::<L>,
            address: PhantomData::<A>,
        }
    }
}

impl<A: Address, L: Level, E: Entry, const N: usize> PageTableMethods<A, L, E, N>
    for PageTable<A, L, E, N>
{
    fn new_with_align(size: usize, align: usize) -> Result<*mut PageTable<A, L, E, N>, Error> {
        assert_eq!(N, L::NUM_ENTRIES);

        let table = unsafe {
            Self::alloc_zeroed(
                Layout::from_size_align(L::TABLE_SIZE * size, L::TABLE_ALIGN * align).unwrap(),
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

    fn new_with_base(base: usize) -> Result<*mut PageTable<A, L, E, N>, Error> {
        let table = base as *mut PageTable<A, L, E, N>;
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
        is_raw: bool,
    ) -> Result<(), Error> {
        let mut phys = phys;
        for guest in guest {
            let phys = phys.next().unwrap();
            match self.set_page(guest, phys, flags, is_raw) {
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
        is_raw: bool,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(guest.address().into());
        if is_raw {
            self.entries[index].set(phys.address(), flags, is_raw)
        } else {
            self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG, is_raw)
        }
    }

    default fn entry<
        S: PageSize,
        F: FnMut(&mut E) -> Result<Option<EntryGuard<'_, E::Inner>>, Error>,
    >(
        &mut self,
        guest: Page<S, A>,
        level: usize,
        no_valid_check: bool,
        mut func: F,
    ) -> Result<(Option<EntryGuard<'_, E::Inner>>, usize), Error> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);
        if level > S::MAP_TABLE_LEVEL {
            return Err(Error::MmInvalidLevel);
        }
        if level != L::THIS_LEVEL {
            return Err(Error::MmInvalidLevel);
        }

        let index = E::index::<L>(guest.address().into());

        if no_valid_check {
            Ok((func(&mut self.entries[index])?, L::THIS_LEVEL))
        } else {
            match self.entries[index].is_valid() {
                true => Ok((func(&mut self.entries[index])?, L::THIS_LEVEL)),
                false => Err(Error::MmNoEntry),
            }
        }
    }

    default fn drop(&mut self) {
        unsafe {
            Self::dealloc(
                self as *mut PageTable<A, L, E, N> as *mut u8,
                Layout::from_size_align(L::TABLE_SIZE, L::TABLE_ALIGN).unwrap(),
            );
        }
    }

    default fn unset_page<S: PageSize>(&mut self, guest: Page<S, A>) {
        let index = E::index::<L>(guest.address().into());
        if self.entries[index].is_valid() {
            let _res = self.entry(guest, S::MAP_TABLE_LEVEL, false, |e| {
                e.clear();
                Ok(None)
            });
        }
    }
}

/// This overrides default PageTableMethods for PageTables with subtable.
/// (L0Table, L1Table, L2Table)
/// PageTableMethods for L3 Table remains unmodified.
impl<A: Address, L: HasSubtable, E: Entry, const N: usize> PageTableMethods<A, L, E, N>
    for PageTable<A, L, E, N>
where
    L::NextLevel: Level,
    [E; L::NextLevel::NUM_ENTRIES]: Sized,
{
    fn entry<S: PageSize, F: FnMut(&mut E) -> Result<Option<EntryGuard<'_, E::Inner>>, Error>>(
        &mut self,
        page: Page<S, A>,
        level: usize,
        no_valid_check: bool,
        mut func: F,
    ) -> Result<(Option<EntryGuard<'_, E::Inner>>, usize), Error> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        if level > S::MAP_TABLE_LEVEL {
            return Err(Error::MmInvalidLevel);
        }
        let index = E::index::<L>(page.address().into());

        if no_valid_check {
            if L::THIS_LEVEL < level {
                // Need to go deeper (recursive)
                match self.subtable::<S>(page) {
                    Ok(subtable) => subtable.entry(page, level, no_valid_check, func),
                    Err(_e) => Ok((None, L::THIS_LEVEL)),
                }
            } else {
                // The page is either LargePage or HugePage
                Ok((func(&mut self.entries[index])?, L::THIS_LEVEL))
            }
        } else {
            match self.entries[index].is_valid() {
                true => {
                    if L::THIS_LEVEL < level {
                        // Need to go deeper (recursive)
                        match self.subtable::<S>(page) {
                            Ok(subtable) => subtable.entry(page, level, no_valid_check, func),
                            Err(_e) => Ok((None, L::THIS_LEVEL)),
                        }
                    } else {
                        // The page is either LargePage or HugePage
                        Ok((func(&mut self.entries[index])?, L::THIS_LEVEL))
                    }
                }
                false => Err(Error::MmNoEntry),
            }
        }
    }

    fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
        is_raw: bool,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(guest.address().into());

        if L::THIS_LEVEL < S::MAP_TABLE_LEVEL {
            self.entries[index].set_with_page_table_flags_via_alloc(index, || {
                let subtable = unsafe {
                    Self::alloc_zeroed(
                        Layout::from_size_align(
                            L::NextLevel::TABLE_SIZE,
                            L::NextLevel::TABLE_ALIGN,
                        )
                        .unwrap(),
                    )
                }
                    as *mut PageTable<A, L::NextLevel, E, { L::NextLevel::NUM_ENTRIES }>;

                if subtable as usize != 0 {
                    let subtable_ptr = subtable
                        as *mut PageTable<A, L::NextLevel, E, { L::NextLevel::NUM_ENTRIES }>;
                    unsafe {
                        let arr: [E; L::NextLevel::NUM_ENTRIES] =
                            core::array::from_fn(|_| E::new());
                        (*subtable_ptr).entries = arr;
                    }
                }

                subtable as usize
            })?;

            // map the page in the subtable (recursive)
            self.subtable_and_set_page(guest, phys, flags, is_raw)
        } else if L::THIS_LEVEL == S::MAP_TABLE_LEVEL {
            // Map page in this level page table
            if is_raw {
                self.entries[index].set(phys.address(), flags, is_raw)
            } else {
                self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG, is_raw)
            }
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
            Self::dealloc(
                self as *mut PageTable<A, L, E, N> as *mut u8,
                Layout::from_size_align(L::TABLE_SIZE, L::TABLE_SIZE).unwrap(),
            );
        }
    }

    fn unset_page<S: PageSize>(&mut self, guest: Page<S, A>) {
        let index = E::index::<L>(guest.address().into());

        if self.entries[index].is_valid() {
            let _res = self.entry(guest, S::MAP_TABLE_LEVEL, false, |e| {
                e.clear();
                Ok(None)
            });
        }
    }
}

impl<A: Address, L: HasSubtable, E: Entry, const N: usize> PageTable<A, L, E, N>
where
    L::NextLevel: Level,
{
    /// Returns the next subtable for the given page in the page table hierarchy.
    fn subtable<S: PageSize>(
        &self,
        page: Page<S, A>,
    ) -> Result<&mut PageTable<A, L::NextLevel, E, N>, Error> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(page.address().into());
        match self.entries[index].subtable(index, L::THIS_LEVEL) {
            Ok(table_addr) => {
                Ok(unsafe { &mut *(table_addr as *mut PageTable<A, L::NextLevel, E, N>) })
            }
            Err(_) => Err(Error::MmSubtableError),
        }
    }

    fn subtable_and_set_page<S: PageSize>(
        &mut self,
        page: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
        is_raw: bool,
    ) -> Result<(), Error> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(page.address().into());
        let table_addr = self.entries[index].subtable(index, L::THIS_LEVEL)?;
        let table = unsafe { &mut *(table_addr as *mut PageTable<A, L::NextLevel, E, N>) };
        table.set_page(page, phys, flags, is_raw)
    }
}
