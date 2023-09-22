use core::mem::MaybeUninit;
use linked_list_allocator::LockedHeap;

use crate::config::RMM_HEAP_SIZE;

static mut HEAP: [MaybeUninit<u8>; RMM_HEAP_SIZE] = [MaybeUninit::uninit(); RMM_HEAP_SIZE];
#[global_allocator]
static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

pub unsafe fn init() {
    ALLOCATOR.lock().init_from_slice(&mut HEAP);
}

pub fn get_used_size() -> usize {
    unsafe { ALLOCATOR.lock().used() }
}
