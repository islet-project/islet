extern crate alloc;

use super::address::{Address, PhysAddr};
use super::error::Error;
use super::guard::EntryGuard;
use super::page::{Page, PageIter, PageSize};

use alloc::alloc::Layout;
use core::marker::PhantomData;
use core::slice::Iter;

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

    fn set(&mut self, addr: PhysAddr, flags: u64) -> Result<(), Error>;
    fn point_to_subtable(&mut self, index: usize, addr: PhysAddr) -> Result<(), Error>;

    // returns EntryGuard which allows accessing what's inside Entry while holding a proper lock.
    fn lock(&self) -> Result<Option<EntryGuard<'_, Self::Inner>>, Error> {
        Err(Error::MmUnimplemented)
    }

    fn index<L: Level>(addr: usize) -> usize;

    // This duplicates with address()
    fn as_subtable(&self, _index: usize, level: usize) -> Result<usize, Error> {
        match self.address(level) {
            Some(addr) => Ok(addr.as_usize()),
            _ => Err(Error::MmInvalidAddr),
        }
    }

    fn points_to_table_or_page(&self) -> bool;
}

/// Safety: the caller must do proper error handling if it's failed to allocate memory
pub trait MemAlloc {
    unsafe fn allocate(&self, layout: Layout) -> *mut u8 {
        alloc::alloc::alloc(layout)
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        alloc::alloc::dealloc(ptr, layout);
    }
}

pub struct DefaultMemAlloc {}

impl MemAlloc for DefaultMemAlloc {}

pub struct PageTable<A, L, E: Entry, const N: usize> {
    entries: [E; N],
    level: PhantomData<L>,
    address: PhantomData<A>,
}

impl<A: Address, L: Level, E: Entry, const N: usize> MemAlloc for PageTable<A, L, E, N> {}
impl<A: Address, L: HasSubtable, E: Entry, const N: usize> MemAlloc for PageTable<A, L, E, N> {}

pub trait PageTableMethods<A: Address, L, E: Entry, const N: usize> {
    /// Sets multiple page table entries
    ///
    /// (input)
    ///    guest : an iterator of target guest addresses to modify their page table entry mapping
    ///    phys  : an iterator of target physical addresses to be mapped
    ///    flags : flags to attach
    fn set_pages<S: PageSize>(
        &mut self,
        guest: PageIter<S, A>,
        phys: PageIter<S, PhysAddr>,
        flags: u64,
    ) -> Result<(), Error>;
    /// Sets a single page table entry
    ///
    /// (input)
    ///    guest : a target guest address to modify its page table entry mapping
    ///    phys  : a target physical address to be mapped
    ///    flags : flags to attach
    fn set_page<S: PageSize>(
        &mut self,
        guest: Page<S, A>,
        phys: Page<S, PhysAddr>,
        flags: u64,
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
    fn entry<
        'a,
        S: PageSize + 'a,
        F: FnMut(&mut E) -> Result<Option<EntryGuard<'_, E::Inner>>, Error>,
    >(
        &'a mut self,
        guest: Page<S, A>,
        level: usize,
        no_valid_check: bool,
        func: F,
    ) -> Result<(Option<EntryGuard<'_, E::Inner>>, usize), Error>;
    /// Traverses page tables from the root and locate the page table at a specific level.
    ///
    /// (input)
    ///    page: a target page to translate
    ///    level: the intended page-table level to reach
    ///
    /// (output)
    ///    if exists,
    ///      A tuple of
    ///        (entry array iterartor, the lastly reached page-table level (usize))
    ///    else,
    ///      None
    fn table_entries<'a, S: PageSize + 'a>(
        &'a self,
        page: Page<S, A>,
        level: usize,
    ) -> Result<(Iter<'a, E>, usize), Error>;
    fn drop(&mut self);
    fn unset_page<S: PageSize>(&mut self, guest: Page<S, A>);
}

impl<A: Address, L: Level, E: Entry, const N: usize> PageTable<A, L, E, N> {
    pub fn new_in(alloc: &dyn MemAlloc) -> Result<&mut PageTable<A, L, E, N>, Error> {
        assert_eq!(N, L::NUM_ENTRIES);
        let table = unsafe {
            alloc.allocate(Layout::from_size_align(L::TABLE_SIZE, L::TABLE_ALIGN).unwrap())
        } as *mut PageTable<A, L, E, N>;

        if table as usize == 0 {
            return Err(Error::MmAllocFail);
        }
        unsafe {
            for entry in (*table).entries.iter_mut() {
                // In general .write is required to avoid attempting to destruct
                // the (uninitialized) previous contents of `entry`,
                // though for the simple Entry with u64 would have worked using
                // (*entry) = E::new();
                // See https://github.com/zakarumych/allocator-api2/blob/main/src/stable/boxed.rs#L905
                let e = E::new();
                core::ptr::copy_nonoverlapping(&e, entry, 1);
                core::mem::forget(e);
            }
            (*table).level = PhantomData::<L>;
            (*table).address = PhantomData::<A>;
            Ok(&mut *table)
        }
    }

    pub fn new_init_in(
        alloc: &dyn MemAlloc,
        mut func: impl FnMut(&mut [E; N]),
    ) -> Result<*mut PageTable<A, L, E, N>, Error> {
        assert_eq!(N, L::NUM_ENTRIES);
        let table = unsafe {
            alloc.allocate(Layout::from_size_align(L::TABLE_SIZE, L::TABLE_ALIGN).unwrap())
        } as *mut PageTable<A, L, E, N>;

        if table as usize == 0 {
            return Err(Error::MmAllocFail);
        }
        unsafe {
            func(&mut (*table).entries);
            (*table).level = PhantomData::<L>;
            (*table).address = PhantomData::<A>;
        }
        Ok(table)
    }
}

impl<A: Address, L: Level, E: Entry, const N: usize> PageTableMethods<A, L, E, N>
    for PageTable<A, L, E, N>
{
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
        self.entries[index].set(phys.address(), flags | S::MAP_EXTRA_FLAG)
    }

    default fn entry<
        'a,
        S: PageSize + 'a,
        F: FnMut(&mut E) -> Result<Option<EntryGuard<'_, E::Inner>>, Error>,
    >(
        &'a mut self,
        guest: Page<S, A>,
        level: usize,
        no_valid_check: bool,
        mut func: F,
    ) -> Result<(Option<EntryGuard<'_, E::Inner>>, usize), Error> {
        assert!(L::THIS_LEVEL == S::MAP_TABLE_LEVEL);
        if level > S::MAP_TABLE_LEVEL {
            return Err(Error::MmInvalidLevel);
        }
        // TODO: remove the level param out of the entry() and don't check this
        if level != L::THIS_LEVEL {
            return Err(Error::MmInvalidLevel);
        }

        // TODO: check if the index is within the total number of entries
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

    default fn table_entries<'a, S: PageSize + 'a>(
        &'a self,
        _page: Page<S, A>,
        _level: usize,
    ) -> Result<(Iter<'a, E>, usize), Error> {
        Ok((self.entries.iter(), L::THIS_LEVEL))
    }

    default fn drop(&mut self) {
        unsafe {
            // FIXME: need to use allocator that is used at new_in()
            let allocator = DefaultMemAlloc {};
            allocator.deallocate(
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
    fn entry<
        'a,
        S: PageSize + 'a,
        F: FnMut(&mut E) -> Result<Option<EntryGuard<'_, E::Inner>>, Error>,
    >(
        &'a mut self,
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

    default fn table_entries<'a, S: PageSize + 'a>(
        &'a self,
        page: Page<S, A>,
        level: usize,
    ) -> Result<(Iter<'a, E>, usize), Error> {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        if level > S::MAP_TABLE_LEVEL {
            return Err(Error::MmInvalidLevel);
        }

        if L::THIS_LEVEL < level {
            match self.subtable::<S>(page) {
                Ok(subtable) => subtable.table_entries(page, level),
                Err(_e) => Ok((self.entries.iter(), L::THIS_LEVEL)),
            }
        } else {
            Ok((self.entries.iter(), L::THIS_LEVEL))
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
            // map the page in the subtable (recursive)
            let subtable = match self.subtable::<S>(guest) {
                Ok(table) => table,
                Err(_) => {
                    let table =
                        PageTable::<A, L::NextLevel, E, { L::NextLevel::NUM_ENTRIES }>::new_in(
                            &DefaultMemAlloc {},
                        )?;
                    self.entries[index]
                        .point_to_subtable(index, PhysAddr::from(core::ptr::from_ref(table)))?;
                    table
                }
            };
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
            let allocator = DefaultMemAlloc {};
            allocator.deallocate(
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
    ) -> Result<&mut PageTable<A, L::NextLevel, E, { L::NextLevel::NUM_ENTRIES }>, Error> {
        assert!(L::THIS_LEVEL < S::MAP_TABLE_LEVEL);

        let index = E::index::<L>(page.address().into());
        match self.entries[index].as_subtable(index, L::THIS_LEVEL) {
            Ok(table_addr) => Ok(unsafe {
                &mut *(table_addr
                    as *mut PageTable<A, L::NextLevel, E, { L::NextLevel::NUM_ENTRIES }>)
            }),
            Err(_) => Err(Error::MmSubtableError),
        }
    }
}
