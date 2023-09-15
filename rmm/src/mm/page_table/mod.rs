pub mod attr;
pub mod entry;

use self::entry::Entry;
use crate::config::PAGE_SIZE;

use vmsa::page_table::{HasSubtable, Level};

// Safety/TODO:
//  - As of now, concurrency safety for RTT and Realm page table is achieved by a big lock.
//  - If we want to use entry-level locking for a better efficiency, several pieces of codes in this file should be modified accordingly.

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
