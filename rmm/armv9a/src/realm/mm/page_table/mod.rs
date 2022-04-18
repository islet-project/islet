use monitor::const_assert_size;
use monitor::mm::page_table::{HasSubtable, Level, PageTable};
use monitor::realm::mm::address::GuestPhysAddr;

use crate::config::PAGE_SIZE;
use entry::Entry;

pub mod entry;
pub mod pte;

/// The Level 0 Table
pub enum L0Table {}
impl Level for L0Table {
    const THIS_LEVEL: usize = 0;
}
impl HasSubtable for L0Table {
    type NextLevel = L1Table;
}

/// The Level 1 Table
pub enum L1Table {}
impl Level for L1Table {
    const THIS_LEVEL: usize = 1;
}
impl HasSubtable for L1Table {
    type NextLevel = L2Table;
}

/// The Level 2 Table
pub enum L2Table {}
impl Level for L2Table {
    const THIS_LEVEL: usize = 2;
}
impl HasSubtable for L2Table {
    type NextLevel = L3Table;
}

/// The Level 3 Table (Doesn't have Subtable!)
pub enum L3Table {}
impl Level for L3Table {
    const THIS_LEVEL: usize = 3;
}

const_assert_size!(PageTable<GuestPhysAddr, L0Table, Entry>, PAGE_SIZE);
const_assert_size!(PageTable<GuestPhysAddr, L1Table, Entry>, PAGE_SIZE);
const_assert_size!(PageTable<GuestPhysAddr, L2Table, Entry>, PAGE_SIZE);
const_assert_size!(PageTable<GuestPhysAddr, L3Table, Entry>, PAGE_SIZE);
