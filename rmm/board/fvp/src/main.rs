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

use monitor::communication::Sender;
use monitor::listen;
use monitor::mainloop::Mainloop;
use monitor::rmi::{self, Receiver};

#[no_mangle]
pub unsafe fn main() -> ! {
    info!(
        "booted on core {:2} with EL{}!",
        cpu::get_cpu_id(),
        helper::regs::current_el()
    );

    let receiver = Receiver::new();
    let sender = receiver.sender();
    let mut mainloop = Mainloop::new(receiver);
    init_instance();
    rmi::gpt::set_event_handler(&mut mainloop);
    rmi::realm::set_event_handler(&mut mainloop);
    rmi::version::set_event_handler(&mut mainloop);

    listen!(mainloop, || {
        warn!("RMM: idle handler called.");
    });

    let ret = mock::boot_complete();
    sender.send(&ret);

    mainloop.run();

    panic!("failed to run the mainloop");
}

fn init_instance() {
    monitor::realm::set_instance(armv9a::realm::registry::Manager::new());
    monitor::smc::set_instance(armv9a::smc::SMC::new());
}

mod mock {
    pub(super) unsafe fn boot_complete() -> [usize; 4] {
        const BOOT_COMPLETE: u64 = 0xC400_01CF;
        const BOOT_SUCCESS: u64 = 0x0;

        let mut ret: [usize; 4] = [0usize; 4];
        core::arch::asm!(
            "smc #0x0",
            inlateout("x0") BOOT_COMPLETE => ret[0],
            inlateout("x1") BOOT_SUCCESS => ret[1],
            out("x2") ret[2],
            out("x3") ret[3],
        );
        ret
    }
}
