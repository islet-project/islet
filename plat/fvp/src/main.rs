#![no_std]
#![no_main]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

#[macro_use]
extern crate log;

mod entry;

use aarch64_cpu::registers::*;
use islet_rmm::allocator;
use islet_rmm::config::PlatformMemoryLayout;
use islet_rmm::cpu;

extern "C" {
    static __RMM_BASE__: u64;
    static __RW_START__: u64;
    static __RW_END__: u64;
    static __RMM_STACK_BASE__: u64;
}

#[no_mangle]
pub unsafe fn main() -> ! {
    info!(
        "booted on core {:2} with EL{}!",
        cpu::get_cpu_id(),
        CurrentEL.read(CurrentEL::EL) as u8
    );

    let layout = unsafe {
        PlatformMemoryLayout {
            rmm_base: &__RMM_BASE__ as *const u64 as u64,
            rw_start: &__RW_START__ as *const u64 as u64,
            rw_end: &__RW_END__ as *const u64 as u64,
            stack_base: &__RMM_STACK_BASE__ as *const u64 as u64,
            uart_phys: 0x1c0c_0000,
        }
    };
    islet_rmm::start(cpu::get_cpu_id(), layout);

    panic!("failed to run the mainloop");
}
