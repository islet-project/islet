#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

#[macro_use]
extern crate log;

mod entry;

use armv9a::allocator;
use armv9a::cpu;
use armv9a::helper;

use monitor;

#[no_mangle]
pub unsafe fn main() -> ! {
    info!(
        "booted on core {:2} with EL{}!",
        cpu::get_cpu_id(),
        helper::regs::current_el()
    );

    let rmi = armv9a::realm::registry::RMI::new();
    let smc = armv9a::smc::SMC::new();
    let rmm = armv9a::rmm::MemoryMap::new();
    let monitor = monitor::Monitor::new(rmi, smc, rmm);
    monitor.boot_complete();
    monitor.run();

    panic!("failed to run the mainloop");
}
