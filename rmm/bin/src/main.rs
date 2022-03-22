#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(const_btree_new)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(global_asm)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]

pub mod aarch64;
pub mod allocator;
pub mod config;
pub mod driver;
pub mod entry;
pub mod panic;
pub mod realm;
pub mod rmi;
pub mod smc;

extern crate alloc;

#[macro_use(bitflags)]
extern crate bitflags;

use rmm_core::communication::Event;
use rmm_core::io::Write as IoWrite;
use rmm_core::mainloop::Mainloop;
use rmm_core::{eprintln, println};

#[no_mangle]
#[allow(unused)]
pub unsafe fn main() -> ! {
    println!(
        "RMM: booted on core {:2} with EL{}!",
        aarch64::cpu::get_cpu_id(),
        aarch64::regs::current_el()
    );

    //TODO remove this test code after normal world setting
    match realm::registry::get(0) {
        Some(vm) => vm,
        _ => realm::registry::new(config::NUM_OF_CPU),
    }
    .lock()
    .switch_to(aarch64::cpu::get_cpu_id());

    let mut mainloop = Mainloop::new(rmi::Receiver::new());

    mainloop.set_event_handler(rmi::Code::Version, |call| {
        println!("RMM: requested version information");
        call.reply(config::ABI_VERSION);
    });

    mainloop.set_event_handler(rmi::Code::GranuleDelegate, |call| {
        println!("RMM: requested granule delegation");
        let cmd = usize::from(smc::Code::MarkRealm);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        call.reply(ret[0]);
    });

    mainloop.set_event_handler(rmi::Code::GranuleUndelegate, |call| {
        println!("RMM: requested granule undelegation");
        let cmd = usize::from(smc::Code::MarkNonSecure);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        call.reply(ret[0]);
    });

    mainloop.set_event_handler(rmi::Code::VMCreate, |call| {
        let num_of_vcpu = call.argument()[0];
        println!("RMM: requested to create VM with {} vcpus", num_of_vcpu);
        let vm = realm::registry::new(num_of_vcpu);
        println!("RMM: create VM {}", vm.lock().id());
        call.reply(vm.lock().id());
    });

    mainloop.set_event_handler(rmi::Code::VMSwitch, |call| {
        let vm = call.argument()[0];
        let vcpu = call.argument()[1];
        println!("RMM: requested to switch to VCPU {} on VM {}", vcpu, vm);
        realm::registry::get(vm).unwrap().lock().switch_to(vcpu);
    });

    mainloop.set_event_handler(rmi::Code::VMResume, |_| { /* Intentionally emptied */ });

    mainloop.set_event_handler(rmi::Code::VMDestroy, |call| {
        let vm = call.argument()[0];
        println!("RMM: requested to destroy VM {}", vm);
        match realm::registry::remove(vm) {
            Ok(_) => call.reply(0),
            Err(_) => call.reply(usize::MAX),
        };
    });

    mainloop.set_event_handler(rmi::Code::Version, |call| {
        println!("RMM: requested version information");
        call.reply(config::ABI_VERSION);
    });

    mainloop.set_default_handler(|call| {
        eprintln!("RMM: no proper rmi handler - code:{:?}", call.code());
    });

    mainloop.set_idle_handler(|| {
        aarch64::rmm_exit();
    });

    mainloop.run();

    panic!("failed to run the mainloop");
}

#[no_mangle]
#[allow(unused)]
pub unsafe fn dummy_main() {
    println!(
        "DUMMY: booted on core {:2} with EL{}!",
        aarch64::cpu::get_cpu_id(),
        aarch64::regs::current_el()
    );
    loop {
        println!("DUMMY: switched!");
        aarch64::asm::hvc(5);
    }
}
