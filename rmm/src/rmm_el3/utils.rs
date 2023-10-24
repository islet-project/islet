use alloc::vec::Vec;
use core::{ptr, slice};
use spinning_top::{Spinlock, SpinlockGuard};

pub(super) fn va_to_vec(ptr: usize, len: usize) -> Vec<u8> {
    let ptr: *const u8 = ptr as *const u8;
    unsafe { slice::from_raw_parts(ptr, len).to_vec() }
}

pub(super) fn vec_to_va(vec: &Vec<u8>, ptr: usize, len: usize) {
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

pub(super) fn get_vector(src: &Spinlock<Vec<u8>>) -> Vec<u8> {
    let guard: SpinlockGuard<'_, _> = src.lock();
    guard.clone()
}
