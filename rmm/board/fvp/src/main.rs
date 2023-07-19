#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

#[macro_use]
extern crate log;

mod entry;
mod memory;

use armv9a::allocator;
use armv9a::cpu;
use armv9a::helper;

use monitor;
use memory::FVPGranuleMap;

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
    let mut mainloop = monitor::event::Mainloop::new();
    let granule_map = FVPGranuleMap::new();
    monitor::rmm::granule::create_gst(granule_map);

    mainloop.boot_complete(smc);
    mainloop.run(&monitor);

    panic!("failed to run the mainloop");
}
