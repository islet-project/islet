pub mod page;
pub mod page_table;
pub mod translation;

#[derive(Debug)]
pub struct MemoryMap;
impl MemoryMap {
    pub fn new() -> &'static MemoryMap {
        &MemoryMap {}
    }
}
impl monitor::rmm::RmmPage for MemoryMap {
    fn map(&self, phys: [usize; 4]) -> Result<(), &str> {
        for addr in phys {
            if addr != 0 {
                translation::set_pages_for_rmi(addr);
            }
        }
        Ok(())
    }
    fn unmap(&self, _phys: [usize; 4]) -> Result<(), &str> {
        // TODO
        Ok(())
    }
}
