use super::address::{align_down, Address};

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
pub struct Page<S: PageSize, A: Address> {
    addr: A,
    size: PhantomData<S>,
}

impl<S: PageSize, A: Address> Page<S, A> {
    /// Return the stored virtual address.
    pub fn address(&self) -> A {
        self.addr
    }

    /// Flushes this page from the TLB of this CPU.
    pub fn flush_from_tlb(&self) {
        unimplemented!()
    }

    /// Returns a Page including the given virtual address.
    /// That means, the address is rounded down to a page size boundary.
    pub fn including_address(addr: A) -> Self {
        Self {
            addr: align_down(addr.into(), S::SIZE).into(),
            size: PhantomData,
        }
    }

    /// Returns a PageIter to iterate from the given first Page to the given last Page (inclusive).
    pub fn range(first: Self, last: Self) -> PageIter<S, A> {
        assert!(first.addr <= last.addr);
        PageIter {
            current: first,
            last,
        }
    }

    pub fn range_with_size(addr: A, size: usize) -> PageIter<S, A> {
        let first_page = Page::<S, A>::including_address(addr);
        let last_page = Page::<S, A>::including_address((addr.into() + size - 1).into());
        Page::range(first_page, last_page)
    }
}

/// An iterator to walk through a range of pages of size S.
pub struct PageIter<S: PageSize, A: Address> {
    current: Page<S, A>,
    last: Page<S, A>,
}

impl<S: PageSize, A: Address> Iterator for PageIter<S, A> {
    type Item = Page<S, A>;

    fn next(&mut self) -> Option<Page<S, A>> {
        if self.current.addr <= self.last.addr {
            let p = self.current;
            self.current.addr += S::SIZE.into();
            Some(p)
        } else {
            None
        }
    }
}
