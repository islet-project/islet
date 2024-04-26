use crate::const_assert_size;
use crate::realm::mm::address::GuestPhysAddr;
use vmsa::page_table::{HasSubtable, Level, PageTable};

use crate::config::PAGE_SIZE;
use entry::Entry;

pub mod entry;
pub mod pte;

pub const STRIDE: usize = 9;

/// Root Table
pub enum RootTable<const L: usize, const N: usize> {}
impl<const L: usize, const N: usize> Level for RootTable<L, N> {
    const THIS_LEVEL: usize = L;
    const TABLE_SIZE: usize = (PAGE_SIZE * N);
    const TABLE_ALIGN: usize = PAGE_SIZE;
    const NUM_ENTRIES: usize = (Self::TABLE_SIZE / core::mem::size_of::<Entry>());
}

impl<const N: usize> HasSubtable for RootTable<0, N> {
    type NextLevel = L1Table;
}

impl<const N: usize> HasSubtable for RootTable<1, N> {
    type NextLevel = L2Table;
}

impl<const N: usize> HasSubtable for RootTable<2, N> {
    type NextLevel = L3Table;
}

/// The Level 0 Table
pub enum L0Table {}
impl Level for L0Table {
    const THIS_LEVEL: usize = 0;
    const TABLE_SIZE: usize = PAGE_SIZE;
    const TABLE_ALIGN: usize = PAGE_SIZE;
    const NUM_ENTRIES: usize = (Self::TABLE_SIZE / core::mem::size_of::<Entry>());
}
impl HasSubtable for L0Table {
    type NextLevel = L1Table;
}

/// The Level 1 Table
pub enum L1Table {}
impl Level for L1Table {
    const THIS_LEVEL: usize = 1;
    const TABLE_SIZE: usize = PAGE_SIZE;
    const TABLE_ALIGN: usize = PAGE_SIZE;
    const NUM_ENTRIES: usize = (Self::TABLE_SIZE / core::mem::size_of::<Entry>());
}
impl HasSubtable for L1Table {
    type NextLevel = L2Table;
}

/// The Level 2 Table
pub enum L2Table {}
impl Level for L2Table {
    const THIS_LEVEL: usize = 2;
    const TABLE_SIZE: usize = PAGE_SIZE;
    const TABLE_ALIGN: usize = PAGE_SIZE;
    const NUM_ENTRIES: usize = (Self::TABLE_SIZE / core::mem::size_of::<Entry>());
}
impl HasSubtable for L2Table {
    type NextLevel = L3Table;
}

/// The Level 3 Table (Doesn't have Subtable!)
pub enum L3Table {}
impl Level for L3Table {
    const THIS_LEVEL: usize = 3;
    const TABLE_SIZE: usize = PAGE_SIZE;
    const TABLE_ALIGN: usize = PAGE_SIZE;
    const NUM_ENTRIES: usize = (Self::TABLE_SIZE / core::mem::size_of::<Entry>());
}

const_assert_size!(PageTable<GuestPhysAddr, L0Table, Entry, { L0Table::NUM_ENTRIES }>, PAGE_SIZE);
const_assert_size!(PageTable<GuestPhysAddr, L1Table, Entry, { L1Table::NUM_ENTRIES }>, PAGE_SIZE);
const_assert_size!(PageTable<GuestPhysAddr, L2Table, Entry, { L2Table::NUM_ENTRIES }>, PAGE_SIZE);
const_assert_size!(PageTable<GuestPhysAddr, L3Table, Entry, { L3Table::NUM_ENTRIES }>, PAGE_SIZE);
