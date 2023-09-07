use crate::mm::page::PageSize;

use super::page_table::attr;
use super::page_table::entry::PTDesc;
use crate::config::PAGE_SIZE;
use crate::helper::bits_in_reg;

#[derive(Clone, Copy)]
/// A 4 KiB page mapped in the L3Table.
pub enum RmmBasePageSize {}
impl PageSize for RmmBasePageSize {
    const SIZE: usize = PAGE_SIZE;
    const MAP_TABLE_LEVEL: usize = 3;
    const MAP_EXTRA_FLAG: u64 = bits_in_reg(PTDesc::TYPE, attr::page_type::TABLE_OR_PAGE);
}
