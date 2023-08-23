pub mod page;
pub mod page_table;
pub mod translation;

use crate::config::PAGE_SIZE;
use crate::helper::asm::dcache_flush;

#[derive(Debug)]
pub struct MemoryMap;
impl MemoryMap {
    pub fn new() -> &'static MemoryMap {
        &MemoryMap {}
    }
}
impl monitor::rmm::RmmPage for MemoryMap {
    fn map(&self, addr: usize, secure: bool) -> bool {
        if addr == 0 {
            warn!("map address is empty");
            return false;
        }
        translation::set_pages_for_rmi(addr, secure);
        dcache_flush(addr, PAGE_SIZE);
        true
    }
    fn unmap(&self, addr: usize) -> bool {
        if addr == 0 {
            warn!("map address is empty");
            return false;
        }
        translation::unset_page_for_rmi(addr);
        true
    }
}
