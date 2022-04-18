use monitor::mm::page::PageSize;

use super::page_table::pte;
use super::translation_granule_4k::RawPTE;
use crate::config::{HUGE_PAGE_SIZE, LARGE_PAGE_SIZE, PAGE_SIZE};
use crate::helper::bits_in_reg;

#[derive(Clone, Copy)]
/// A 4 KiB page mapped in the L3Table.
pub enum BasePageSize {}
impl PageSize for BasePageSize {
    const SIZE: usize = PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 3;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(RawPTE::TYPE, pte::page_type::TABLE_OR_PAGE);
    const TABLE_SIZE: usize = PAGE_SIZE;
}

#[derive(Clone, Copy)]
/// A 2 MiB page mapped in the L2Table.
pub enum LargePageSize {}
impl PageSize for LargePageSize {
    const SIZE: usize = LARGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 2;
    const MAP_EXTRA_FLAG: u64 = 0;
    const TABLE_SIZE: usize = PAGE_SIZE;
}

#[derive(Clone, Copy)]
/// A 1 GiB page mapped in the L1Table.
pub enum HugePageSize {}
impl PageSize for HugePageSize {
    const SIZE: usize = HUGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 1;
    const MAP_EXTRA_FLAG: u64 = 0;
    const TABLE_SIZE: usize = PAGE_SIZE;
}
