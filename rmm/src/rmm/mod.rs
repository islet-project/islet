pub mod address;
#[macro_use]
pub mod granule;
pub mod page;
pub mod page_table;
pub mod translation;

use crate::asm::dcache_flush;
use crate::config::PAGE_SIZE;

pub type PageMap = &'static dyn RmmPage;

pub trait RmmPage {
    fn map(&self, phys: usize, secure: bool) -> bool;
    fn unmap(&self, phys: usize) -> bool;
}

#[derive(Debug)]
pub struct MemoryMap;
impl MemoryMap {
    pub fn new() -> &'static MemoryMap {
        &MemoryMap {}
    }
}

impl RmmPage for MemoryMap {
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

/// Call `rmm_exit` within `exception/vectors.s` and jumps to EL1.
///
/// Currently, this function gets [0usize; 3] as an argument to initialize
/// x0, x1 and x2 registers.
///
/// When an exception occurs and the flow comes back to EL2 through `rmm_enter`,
/// x0, x1 and x2 registers might be changed to contain additional information
/// set from `handle_lower_exception`.
/// These are the return values of this function.
/// The return value encodes: [rmi::RET_XXX, ret_val1, ret_val2]
/// In most cases, the function returns [rmi::RET_SUCCESS, _, _]
/// pagefault returns [rmi::RET_PAGE_FAULT, faulted address, _]
pub unsafe fn rmm_exit(args: [usize; 4]) -> [usize; 4] {
    let mut ret: [usize; 4] = [0usize; 4];

    core::arch::asm!(
        "bl rmm_exit",
        inlateout("x0") args[0] => ret[0],
        inlateout("x1") args[1] => ret[1],
        inlateout("x2") args[2] => ret[2],
        inlateout("x3") args[3] => ret[3],
    );

    ret
}
