#![no_std]
#![no_main]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

#[macro_use]
extern crate log;

mod entry;
mod plat;

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
pub unsafe fn main(x0: u64, x1: u64, x2: u64, x3: u64) -> ! {
    let cpuid: usize = x0 as usize;
    // Do not print here until MMU is turned on, except on cpu0.
    // Cores other than cpu0 at this point are still in its mmu
    // off state with d-cache disabled and i-cache enabled.
    // This may cause incosistencies between cpus with mmu enabled
    // and cpus with mmu disabled.
    // Logging involves buffer allocation and its internal data struct
    // for heap (linked_list) could be corrupted due to the reason above.
    if x0 == 0 {
        info!(
            "boot args: x0:0x{:X}, x1:0x{:X}, x2:0x{:X}, x3:0x{:X}",
            x0, x1, x2, x3
        );
    }

    if cpuid != cpu::get_cpu_id() {
        panic!(
            "x0:{:X} != cpu::get_cput_id()(=={:X})",
            cpuid,
            cpu::get_cpu_id()
        );
    }
    let layout = unsafe {
        PlatformMemoryLayout {
            rmm_base: &__RMM_BASE__ as *const u64 as u64,
            rw_start: &__RW_START__ as *const u64 as u64,
            rw_end: &__RW_END__ as *const u64 as u64,
            stack_base: &__RMM_STACK_BASE__ as *const u64 as u64,
            uart_phys: plat::UART_BASE as u64,
            el3_shared_buf: x3,
        }
    };
    islet_rmm::start(cpuid, layout);

    panic!("failed to run the mainloop");
}
