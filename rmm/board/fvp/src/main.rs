#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(naked_functions)]

extern crate alloc;

mod driver;
mod entry;

use armv9a::aarch64;
use armv9a::allocator;
use armv9a::config;
use armv9a::realm;
use armv9a::rmi;
use armv9a::smc;

use monitor::communication::Event;
use monitor::io::Write as IoWrite;
use monitor::mainloop::Mainloop;
use monitor::{eprintln, println};

#[no_mangle]
#[allow(unused)]
pub unsafe fn main() -> ! {
    println!(
        "RMM: booted on core {:2} with EL{}!",
        aarch64::cpu::get_cpu_id(),
        aarch64::regs::current_el()
    );

    let mut mainloop = Mainloop::new(rmi::Receiver::new());

    mainloop.set_event_handler(rmi::Code::Version, |call| {
        println!("RMM: requested version information");
        call.reply(config::ABI_VERSION);
    });

    mainloop.set_event_handler(rmi::Code::GranuleDelegate, |call| {
        let cmd = usize::from(smc::Code::MarkRealm);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        //println!("RMM: requested granule delegation {:X?}", arg);
        call.reply(ret[0]);
    });

    mainloop.set_event_handler(rmi::Code::GranuleUndelegate, |call| {
        let cmd = usize::from(smc::Code::MarkNonSecure);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        //println!("RMM: requested granule undelegation {:X?}", arg);
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
        if let Some(vcpu) = realm::vcpu::current() {
            if vcpu.is_vm_dead() {
                vcpu.from_current()
            } else {
                aarch64::rmm_exit();
            }
        }
    });

    mainloop.run();

    panic!("failed to run the mainloop");
}
