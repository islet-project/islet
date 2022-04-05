#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
pub mod panic;

#[naked]
#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn bootstrap_entry() {
    #![allow(unsupported_naked_functions)]
    //vm_create
    llvm_asm! {
        "
        smc 0x0
        "
        : : "{x0}"(0xc000_0003 as usize), "{x1}"(1) : : "volatile"
    }
    //vm_switch
    llvm_asm! {
        "
        smc 0x0
        "
        : : "{x0}"(0xc000_0004 as usize) "{x1}"(0) : : "volatile"
    }
    loop {}
}
