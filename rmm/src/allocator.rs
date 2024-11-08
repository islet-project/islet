use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;
use linked_list_allocator::LockedHeap;

use crate::config::RMM_HEAP_SIZE;

static mut HEAP: [MaybeUninit<u8>; RMM_HEAP_SIZE] = [MaybeUninit::uninit(); RMM_HEAP_SIZE];
#[global_allocator]
static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initializes the global allocator with a heap backed by the `HEAP` array.
///
/// # Safety
///
/// - This function must be called exactly once before any memory allocation occurs.
///   Calling it multiple times or after allocations have started can lead to undefined behavior.
pub unsafe fn init() {
    ALLOCATOR.lock().init_from_slice(&mut *addr_of_mut!(HEAP));
}

pub fn get_used_size() -> usize {
    unsafe { ALLOCATOR.lock().used() }
}
