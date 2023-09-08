#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

#[macro_use]
extern crate log;

mod entry;

use islet_rmm::allocator;
use islet_rmm::cpu;

use islet_rmm;

#[no_mangle]
pub unsafe fn main() -> ! {
    info!(
        "booted on core {:2} with EL{}!",
        cpu::get_cpu_id(),
        armv9a::regs::current_el()
    );

    let rmi = islet_rmm::realm::registry::RMI::new();
    let rmm = islet_rmm::rmm::MemoryMap::new();
    islet_rmm::rmm::granule::create_granule_status_table();
    let monitor = islet_rmm::Monitor::new(rmi, rmm);
    let mut mainloop = islet_rmm::event::Mainloop::new();
    mainloop.boot_complete();
    mainloop.run(&monitor);

    panic!("failed to run the mainloop");
}
