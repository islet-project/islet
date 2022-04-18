extern crate alloc;

use super::PageTable;
use crate::config::PAGE_SIZE;

pub fn alloc<T>(num: usize) -> Result<*mut PageTable<T>, ()> {
    let ptr = unsafe {
        alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(PAGE_SIZE * num, PAGE_SIZE * num).unwrap(),
        )
    };
    assert_eq!(
        (ptr as usize) % PAGE_SIZE,
        0,
        "Physical address is not on a {:#X} boundary (paddr = {:#X})",
        PAGE_SIZE,
        ptr as usize
    );
    Ok(ptr as *mut PageTable<T>)
}
