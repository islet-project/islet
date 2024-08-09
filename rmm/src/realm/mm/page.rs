use super::attribute::page_type;
use super::stage2_tte::S2TTE;
use super::table_level::{L1Table, L2Table, L3Table};
use crate::config::{HUGE_PAGE_SIZE, LARGE_PAGE_SIZE, PAGE_SIZE};
use vmsa::page::PageSize;
use vmsa::page_table::Level;

use armv9a::bits_in_reg;

#[derive(Clone, Copy)]
/// A 4 KiB page mapped in the L3Table.
pub enum BasePageSize {}
impl PageSize for BasePageSize {
    const SIZE: usize = PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = L3Table::THIS_LEVEL;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(S2TTE::TYPE, page_type::TABLE_OR_PAGE);
}

#[derive(Clone, Copy)]
/// A 2 MiB page mapped in the L2Table.
pub enum LargePageSize {}
impl PageSize for LargePageSize {
    const SIZE: usize = LARGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = L2Table::THIS_LEVEL;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(S2TTE::TYPE, page_type::BLOCK);
}

#[derive(Clone, Copy)]
/// A 1 GiB page mapped in the L1Table.
pub enum HugePageSize {}
impl PageSize for HugePageSize {
    const SIZE: usize = HUGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = L1Table::THIS_LEVEL;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(S2TTE::TYPE, page_type::BLOCK);
}
