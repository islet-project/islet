extern crate alloc;

use super::page_table::PageTable;
use super::PAGE_SIZE;

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

#[cfg(test)]
pub mod test {
    use crate::virt::mm::page_table::{L1Table, L2Table, PageTable};
    use crate::virt::mm::pgtlb_allocator;
    use crate::virt::mm::PAGE_SIZE;
    use core::mem;
    #[test]
    fn test_alloc_table() {
        let root: Result<*mut PageTable<L1Table>, ()> =
            pgtlb_allocator::allocate_tables(1, PAGE_SIZE * 2);
        assert!(root.is_ok());
        let root: &mut PageTable<L1Table> = unsafe { mem::transmute(root.unwrap()) };
        let subtable: *mut PageTable<L2Table> =
            pgtlb_allocator::allocate_tables(1, PAGE_SIZE).unwrap();
    }
}
