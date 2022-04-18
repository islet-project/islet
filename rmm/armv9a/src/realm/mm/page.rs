use monitor::mm::page::{Address, Page, PageSize};
use monitor::mm::page_table;

use super::page_table::pte;
use super::translation_granule_4k::{RawGPA, RawPTE};
use crate::config::{HUGE_PAGE_SIZE, LARGE_PAGE_SIZE, PAGE_SIZE};
use crate::helper::bits_in_reg;

pub fn table_index<S: PageSize, A: Address, L: page_table::Level>(page: Page<S, A>) -> usize {
    assert!(L::THIS_LEVEL <= S::MAP_TABLE_LEVEL);
    match L::THIS_LEVEL {
        0 => RawGPA::from(page.address().into()).get_masked_value(RawGPA::L0Index) as usize,
        1 => RawGPA::from(page.address().into()).get_masked_value(RawGPA::L1Index) as usize,
        2 => RawGPA::from(page.address().into()).get_masked_value(RawGPA::L2Index) as usize,
        3 => RawGPA::from(page.address().into()).get_masked_value(RawGPA::L3Index) as usize,
        _ => panic!(),
    }
}

#[derive(Clone, Copy)]
/// A 4 KiB page mapped in the L3Table.
pub enum BasePageSize {}
impl PageSize for BasePageSize {
    const SIZE: usize = PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 3;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(RawPTE::TYPE, pte::page_type::TABLE_OR_PAGE);
}

#[derive(Clone, Copy)]
/// A 2 MiB page mapped in the L2Table.
pub enum LargePageSize {}
impl PageSize for LargePageSize {
    const SIZE: usize = LARGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 2;
    const MAP_EXTRA_FLAG: u64 = 0;
}

#[derive(Clone, Copy)]
/// A 1 GiB page mapped in the L1Table.
pub enum HugePageSize {}
impl PageSize for HugePageSize {
    const SIZE: usize = HUGE_PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 1;
    const MAP_EXTRA_FLAG: u64 = 0;
}
