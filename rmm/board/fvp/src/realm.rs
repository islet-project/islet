use monitor::communication::{Error, ErrorKind};
use monitor::io::Write as IoWrite;
use monitor::mainloop::Mainloop;
use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::{eprintln, println};

use armv9a::config::PAGE_SIZE;
use armv9a::helper;
use armv9a::realm;
use armv9a::realm::mm::page_table_entry::{pte_access_perm, pte_mem_attr};
use armv9a::realm::mm::translation_granule_4k::RawPTE;
use armv9a::smc;

use crate::rmi;

pub fn rmm_exit() -> [usize; 3] {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            if vcpu.is_vm_dead() {
                vcpu.from_current();
            } else {
                return helper::rmm_exit([0; 3]);
            }
        }
        [0, 0, 0]
    }
}

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    mainloop.set_event_handler(rmi::Code::VMCreate, |call| {
        let num_of_vcpu = call.argument()[0];
        println!("RMM: requested to create VM with {} vcpus", num_of_vcpu);
        let vm = realm::registry::new(num_of_vcpu);
        println!("RMM: create VM {}", vm.lock().id());
        call.reply(rmi::RET_SUCCESS).unwrap();
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
            Ok(_) => call.reply(rmi::RET_SUCCESS),
            Err(_) => call.reply(rmi::RET_FAIL),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });

    mainloop.set_event_handler(rmi::Code::VMDestroy, |call| {
        let vm = call.argument()[0];
        println!("RMM: requested to destroy VM {}", vm);
        match realm::registry::remove(vm) {
            Ok(_) => call.reply(rmi::RET_SUCCESS),
            Err(_) => call.reply(rmi::RET_FAIL),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });

    mainloop.set_event_handler(rmi::Code::VMRun, |call| {
        println!("RMM: requested to jump to EL1");
        let ret = rmm_exit();

        match ret[0] {
            rmi::RET_SUCCESS => call.reply(rmi::RET_SUCCESS),
            rmi::RET_PAGE_FAULT => {
                call.reply(rmi::RET_PAGE_FAULT).unwrap();
                call.reply(ret[1])
            }
            _ => Err(Error::new(ErrorKind::Unsupported)),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
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

        let cmd = usize::from(smc::Code::MarkRealm);
        let mut arg = [phys, 0, 0, 0];
        let mut remain = size;
        while remain > 0 {
            //TODO change to use dtb
            if arg[0] >= 0x4000_0000 {
                let ret = smc::call(cmd, arg)[0];
                if ret != 0 {
                    //Just show a warn message not return fail
                    eprintln!("RMM: failed to set GPT {:X}", arg[0]);
                }
            }
            arg[0] += PAGE_SIZE;
            remain -= PAGE_SIZE;
        }

        call.reply(rmi::RET_SUCCESS)
            .err()
            .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
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

        //TODO change GPT to nonsecure
        //TODO zeroize memory

        call.reply(rmi::RET_SUCCESS)
            .err()
            .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
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
                call.reply(rmi::RET_SUCCESS)
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
                call.reply(rmi::RET_SUCCESS)
            }
            _ => call.reply(rmi::RET_FAIL),
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
                call.reply(rmi::RET_SUCCESS).unwrap();
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
                call.reply(rmi::RET_SUCCESS).unwrap();
                call.reply(value as usize)
            }
            _ => call.reply(rmi::RET_FAIL),
        }
        .err()
        .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });
}
