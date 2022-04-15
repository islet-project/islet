#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(naked_functions)]

extern crate alloc;

mod driver;
mod entry;

use armv9a::allocator;
use armv9a::config;
use armv9a::cpu;
use armv9a::helper;
use armv9a::rmi;

use monitor::communication::Event;
use monitor::io::Write as IoWrite;
use monitor::mainloop::Mainloop;
use monitor::{eprintln, println};

#[no_mangle]
pub unsafe fn main() -> ! {
    println!(
        "RMM: booted on core {:2} with EL{}!",
        cpu::get_cpu_id(),
        helper::regs::current_el()
    );

    let mut mainloop = Mainloop::new(rmi::Receiver::new());

    rmi::gpt::set_event_handler(&mut mainloop);
    rmi::realm::set_event_handler(&mut mainloop);
    rmi::version::set_event_handler(&mut mainloop);

    mainloop.set_default_handler(|call| {
        eprintln!("RMM: no proper rmi handler - code:{:?}", call.code());
    });

    mainloop.run();

    panic!("failed to run the mainloop");
}
