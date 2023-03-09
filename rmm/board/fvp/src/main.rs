#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

extern crate alloc;

#[macro_use]
extern crate log;

mod driver;
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

    init_instance();

    let monitor = monitor::Monitor::new();
    monitor.boot_complete();
    monitor.run();

    panic!("failed to run the mainloop");
}

fn init_instance() {
    monitor::realm::set_instance(armv9a::realm::registry::Manager::new());
    monitor::smc::set_instance(armv9a::smc::SMC::new());
}
