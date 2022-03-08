use super::page_table::{L0Table, L1Table, L2Table, L3Table};
use super::page_table::{LevelHierarchy, PageTable, PageTableLevel, TranslationGranule};
use super::page_table_entry::{BasePageSize, HugePageSize, LargePageSize, PageSize};
use super::page_table_entry::{Page, PageTableEntryFlags};
use crate::config::PAGE_BITS;
use crate::define_mask;

/// Number of bits of the index in each table level.
const PAGE_MAP_BITS: usize = 9;

/// A mask where PAGE_MAP_BITS are set to calculate a table index.
const PAGE_MAP_MASK: usize = 0x1FF;

/// The Level 0 Table
impl PageTableLevel for L0Table {
    const THIS_LEVEL: usize = 0;
}

impl LevelHierarchy for L0Table {
    type NextLevel = L1Table;
}

/// The Level 1 Table (can map 1 GiB pages)
impl PageTableLevel for L1Table {
    const THIS_LEVEL: usize = 1;
}

impl LevelHierarchy for L1Table {
    type NextLevel = L2Table;
}

/// The Level 2 Table (can map 2 MiB pages)
impl PageTableLevel for L2Table {
    const THIS_LEVEL: usize = 2;
}

impl LevelHierarchy for L2Table {
    type NextLevel = L3Table;
}

/// The Level 3 Table (can map 4 KiB pages)
impl PageTableLevel for L3Table {
    const THIS_LEVEL: usize = 3;
}

impl<L: PageTableLevel> TranslationGranule for PageTable<L> {
    // L0: bits[47:39]
    // L1: bits[38:30]
    // L2: bits[29:21]
    // L3: bits[20:12]
    default fn table_index<S: PageSize>(&self, page: Page<S>) -> usize {
        assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
        page.address().as_usize() >> PAGE_BITS >> (3 - L::THIS_LEVEL) * PAGE_MAP_BITS
            & PAGE_MAP_MASK
    }

    // https://armv8-ref.codingbelief.com/en/chapter_d4/d43_1_vmsav8-64_translation_table_descriptor_formats.html
    // output address for 4KB granule:
    //   - block descriptor block(bits[1:0] == 0b01) :
    //      . level1: bits[47:30]
    //      . level2: bits[47:21]
    //   - table descriptor (bits[1:0] == 0b11)
    //      . level0 ~ level2: bits[47:12]
    //   - page descriptor (bits[1:0] = 0b11)
    //      . level3:  bits[47:12]
    default fn table_addr_mask(&self) -> usize {
        define_mask!(47, 12)
    }
    default fn page_addr_mask(&self) -> usize {
        0x0
    }
}

impl TranslationGranule for PageTable<L1Table> {
    fn page_addr_mask(&self) -> usize {
        define_mask!(47, 30)
    }
}

impl TranslationGranule for PageTable<L2Table> {
    fn page_addr_mask(&self) -> usize {
        define_mask!(47, 21)
    }
}

impl TranslationGranule for PageTable<L3Table> {
    fn table_addr_mask(&self) -> usize {
        0x0
    }

    fn page_addr_mask(&self) -> usize {
        define_mask!(47, 12)
    }
}

/// A 4 KiB page mapped in the L3Table.
impl PageSize for BasePageSize {
    const SIZE: usize = 4096;
    const MAP_TABLE_LEVEL: usize = 3;
    const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::TABLE_OR_PAGE_DESC;
}

/// A 2 MiB page mapped in the L2Table.
impl PageSize for LargePageSize {
    const SIZE: usize = 2 * 1024 * 1024;
    const MAP_TABLE_LEVEL: usize = 2;
    const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A 1 GiB page mapped in the L1Table.
impl PageSize for HugePageSize {
    const SIZE: usize = 1024 * 1024 * 1024;
    const MAP_TABLE_LEVEL: usize = 1;
    const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}
