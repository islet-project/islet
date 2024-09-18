use alloc::vec::Vec;
use core::{ops::Range, ptr, slice};
use spinning_top::{Spinlock, SpinlockGuard};

pub(super) fn va_to_vec(ptr: usize, len: usize) -> Vec<u8> {
    let ptr: *const u8 = ptr as *const u8;
    unsafe { slice::from_raw_parts(ptr, len).to_vec() }
}

pub(super) fn vec_to_va(vec: &[u8], ptr: usize, len: usize) {
    // safety check
    if vec.len() > len {
        panic!("Vector too long");
    }

    let v_ptr = vec.as_ptr();
    let ptr = ptr as *mut u8;
    unsafe {
        ptr::copy(v_ptr, ptr, vec.len());
    }
}

pub(super) fn set_vector(src: Vec<u8>, dst: &Spinlock<Vec<u8>>) {
    let mut guard: SpinlockGuard<'_, _> = dst.lock();
    guard.clear();
    guard.extend_from_slice(&src);
}

// it's up to the caller of this function to make sure dst won't go out of bounds
pub(super) fn set_array<const N: usize>(
    smc_ret: [usize; 8],
    range: Range<usize>,
    dst: &Spinlock<[u8; N]>,
) {
    let mut guard: SpinlockGuard<'_, _> = dst.lock();

    let len = core::mem::size_of::<usize>();
    for (i, reg) in smc_ret[range].iter().enumerate() {
        guard[i * len..i * len + len].copy_from_slice(reg.to_ne_bytes().as_slice());
    }
}

pub(super) fn get_spinlock<T: Clone>(src: &Spinlock<T>) -> T {
    let guard: SpinlockGuard<'_, _> = src.lock();
    guard.clone()
}
