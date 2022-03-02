pub mod address;
pub mod page_table;
pub mod page_table_entry;
pub mod pgtlb_allocator;
pub mod stage2_translation;
pub mod translation_granule_4k;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_BITS: usize = 12;
