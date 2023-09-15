use super::page_table::{attr, entry::PTDesc};
use crate::config::PAGE_SIZE;

use armv9a::bits_in_reg;
use vmsa::page::PageSize;

/// A 4 KiB page mapped in the L3Table.
#[derive(Clone, Copy)]
pub enum BasePageSize {}
impl PageSize for BasePageSize {
    const SIZE: usize = PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 3;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(PTDesc::TYPE, attr::page_type::TABLE_OR_PAGE);
}
