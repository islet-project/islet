use monitor::io::Write as IoWrite;
use monitor::mainloop::Mainloop;
use monitor::{eprintln, println};

use armv9a::helper;
use armv9a::realm;

use crate::rmi;

pub fn rmm_exit() {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            if vcpu.is_vm_dead() {
                vcpu.from_current()
            } else {
                helper::rmm_exit();
            }
        }
    }
}

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    mainloop.set_event_handler(rmi::Code::VMCreate, |call| {
        let num_of_vcpu = call.argument()[0];
        println!("RMM: requested to create VM with {} vcpus", num_of_vcpu);
        let vm = realm::registry::new(num_of_vcpu);
        println!("RMM: create VM {}", vm.lock().id());
        call.reply(vm.lock().id())
            .err()
            .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });

    mainloop.set_event_handler(rmi::Code::VMSwitch, |call| {
        let vm = call.argument()[0];
        let vcpu = call.argument()[1];
        println!("RMM: requested to switch to VCPU {} on VM {}", vcpu, vm);
        match realm::registry::get(vm).unwrap().lock().switch_to(vcpu) {
            Ok(_) => call.reply(0),
            Err(_) => call.reply(usize::MAX),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });

    mainloop.set_event_handler(rmi::Code::VMResume, |_| { /* Intentionally emptied */ });

    mainloop.set_event_handler(rmi::Code::VMDestroy, |call| {
        let vm = call.argument()[0];
        println!("RMM: requested to destroy VM {}", vm);
        match realm::registry::remove(vm) {
            Ok(_) => call.reply(0),
            Err(_) => call.reply(usize::MAX),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });

    mainloop.set_idle_handler(|| {
        rmm_exit();
    });
}
