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
    let rmm = armv9a::rmm::MemoryMap::new();
    monitor::rmm::granule::create_granule_status_table();
    let monitor = monitor::Monitor::new(rmi, rmm);
    let mut mainloop = monitor::event::Mainloop::new();
    mainloop.boot_complete();
    mainloop.run(&monitor);

    panic!("failed to run the mainloop");
}
