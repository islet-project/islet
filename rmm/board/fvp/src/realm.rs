use monitor::io::Write as IoWrite;
use monitor::mainloop::Mainloop;
use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::{eprintln, println};

use armv9a::helper;
use armv9a::realm;
use armv9a::realm::mm::page_table_entry::{pte_access_perm, pte_mem_attr};
use armv9a::realm::mm::translation_granule_4k::RawPTE;

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
        //TODO remove unwrap
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

    mainloop.set_event_handler(rmi::Code::VMMapMemory, |call| {
        let vm = call.argument()[0];
        let guest = call.argument()[1];
        let phys = call.argument()[2];
        let size = call.argument()[3];

        let flags = helper::bits_in_reg(RawPTE::ATTR, pte_mem_attr::NORMAL)
            | helper::bits_in_reg(RawPTE::S2AP, pte_access_perm::RW);

        //TODO remove unwrap
        realm::registry::get(vm)
            .unwrap()
            .lock()
            .page_table
            .lock()
            .set_pages(
                GuestPhysAddr::from(guest),
                PhysAddr::from(phys),
                size,
                flags as usize,
            );
    });

    mainloop.set_event_handler(rmi::Code::VMUnmapMemory, |call| {
        let vm = call.argument()[0];
        let guest = call.argument()[1];
        let size = call.argument()[2];

        //TODO remove unwrap
        realm::registry::get(vm)
            .unwrap()
            .lock()
            .page_table
            .lock()
            .unset_pages(GuestPhysAddr::from(guest), size);
    });

    mainloop.set_event_handler(rmi::Code::VMSetReg, |call| {
        let vm = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        let value = call.argument()[3];
        match register {
            0..=30 => {
                //TODO remove unwrap
                realm::registry::get(vm)
                    .unwrap()
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .unwrap()
                    .lock()
                    .context
                    .gp_regs[register] = value as u64;
                call.reply(0)
            }
            31 => {
                //TODO remove unwrap
                realm::registry::get(vm)
                    .unwrap()
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .unwrap()
                    .lock()
                    .context
                    .elr = value as u64;
                call.reply(0)
            }
            _ => call.reply(usize::MAX),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });

    mainloop.set_event_handler(rmi::Code::VMGetReg, |call| {
        let vm = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        match register {
            0..=30 => {
                //TODO remove unwrap
                let value = realm::registry::get(vm)
                    .unwrap()
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .unwrap()
                    .lock()
                    .context
                    .gp_regs[register];
                call.reply(value as usize)
            }
            31 => {
                //TODO remove unwrap
                let value = realm::registry::get(vm)
                    .unwrap()
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .unwrap()
                    .lock()
                    .context
                    .elr;
                call.reply(value as usize)
            }
            _ => call.reply(usize::MAX),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });
}
