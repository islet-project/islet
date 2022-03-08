extern crate alloc;

use super::page_table::PageTable;
use crate::config::PAGE_SIZE;

pub fn allocate_tables<T>(num: usize, align: usize) -> Result<*mut PageTable<T>, ()> {
    let ptr = unsafe {
        alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(PAGE_SIZE * num, align).unwrap(),
        )
    };
    assert_eq!(
        (ptr as usize) % align,
        0,
        "Physical address is not on a {:#X} boundary (paddr = {:#X})",
        align,
        ptr as usize
    );
    Ok(ptr as *mut PageTable<T>)
}
