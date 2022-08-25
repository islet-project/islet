use monitor::communication::{Error, ErrorKind};
use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::{listen, mainloop::Mainloop};

use crate::config::PAGE_SIZE;
use crate::helper;
use crate::realm;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::translation_granule_4k::RawPTE;
use crate::smc;

use crate::rmi;

pub fn rmm_exit() -> [usize; 4] {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            if vcpu.is_vm_dead() {
                vcpu.from_current();
            } else {
                vcpu.vm.upgrade().map(|vm| {
                    vm.lock().page_table.lock().clean();
                });
                return helper::rmm_exit([0; 4]);
            }
        }
        [0, 0, 0, 0]
    }
}

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::VMCreate, |call| {
        info!("received VMCreate");
        let vm = realm::registry::new();
        info!("create VM {}", vm.lock().id());
        call.reply(rmi::RET_SUCCESS)?;
        call.reply(vm.lock().id())?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VCPUCreate, |call| {
        let vm = call.argument()[0];
        // let vcpu = call.argument()[1];
        debug!("received VCPUCreate for VM {}", vm);
        match realm::registry::get(vm)
            .ok_or("Not exist VM")?
            .lock()
            .create_vcpu()
        {
            Ok(vcpuid) => {
                call.reply(rmi::RET_SUCCESS)?;
                call.reply(vcpuid)
            }
            Err(_) => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMDestroy, |call| {
        let vm = call.argument()[0];
        info!("received VMDestroy VM {}", vm);
        match realm::registry::remove(vm) {
            Ok(_) => call.reply(rmi::RET_SUCCESS),
            Err(_) => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMRun, |call| {
        let vm = call.argument()[0];
        let vcpu = call.argument()[1];
        let incr_pc = call.argument()[2];
        debug!(
            "received VMRun VCPU {} on VM {} PC_INCR {}",
            vcpu, vm, incr_pc
        );
        if incr_pc == 1 {
            realm::registry::get(vm)
                .ok_or("Not exist VM")?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or("Not exist VCPU")?
                .lock()
                .context
                .elr += 4;
        }

        realm::registry::get(vm)
            .ok_or("Not exist VM")?
            .lock()
            .switch_to(vcpu)?;

        trace!("Switched to VCPU {} on VM {}", vcpu, vm);
        let ret = rmm_exit();

        match ret[0] {
            rmi::RET_SUCCESS => call.reply(rmi::RET_SUCCESS),
            rmi::RET_EXCEPTION_TRAP | rmi::RET_EXCEPTION_IRQ => {
                call.reply(ret[0]).or(Err("RMM failed to reply."))?;
                call.reply(ret[1]);
                call.reply(ret[2]);
                call.reply(ret[3])
            }
            _ => Err(Error::new(ErrorKind::Unsupported)),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMMapMemory, |call| {
        let vm = call.argument()[0];
        let guest = call.argument()[1];
        let phys = call.argument()[2];
        let size = call.argument()[3];
        // prot: bits[0] : writable, bits[1] : fault on exec, bits[2] : device
        let prot = call.argument()[4]; // bits[3]
        debug!(
            "received MapMemory to VM {} {:#X} -> {:#X} size:{:#X} prot:{:#X}",
            vm, guest, phys, size, prot
        );

        let mut flags = 0;
        // TODO:  define bit mask
        if prot & 0b1 == 0b1 {
            flags |= helper::bits_in_reg(RawPTE::S2AP, pte::permission::RW);
        } else {
            flags |= helper::bits_in_reg(RawPTE::S2AP, pte::permission::RO);
        }
        if prot & 0b100 == 0b100 {
            flags |= helper::bits_in_reg(RawPTE::ATTR, pte::attribute::DEVICE_NGNRE)
        } else {
            flags |= helper::bits_in_reg(RawPTE::ATTR, pte::attribute::NORMAL)
        }

        // TODO: shared between NS and Linux Ream for GITS_CBASER
        if guest >= 0x40C2_0000 && guest < 0x40C4_0000 {
            flags |= helper::bits_in_reg(RawPTE::NS, 0b1);
        }

        realm::registry::get(vm)
            .ok_or("Not exist VM")?
            .lock()
            .page_table
            .lock()
            .set_pages(
                GuestPhysAddr::from(guest),
                PhysAddr::from(phys),
                size,
                flags as usize,
            );

        // TODO: shared between NS and Linux Ream for GITS_CBASER
        if guest >= 0x40C2_0000 && guest < 0x40C4_0000 {
            return Ok(());
        }

        let cmd = usize::from(smc::Code::MarkRealm);
        let mut arg = [phys, 0, 0, 0];
        let mut remain = size;
        while remain > 0 {
            //TODO change to use dtb
            if arg[0] >= 0x4000_0000 {
                let ret = smc::call(cmd, arg)[0];
                if ret != 0 {
                    //Just show a warn message not return fail
                    warn!("failed to set GPT {:X}", arg[0]);
                }
            }
            arg[0] += PAGE_SIZE;
            remain -= PAGE_SIZE;
        }

        call.reply(rmi::RET_SUCCESS)?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMUnmapMemory, |call| {
        let vm = call.argument()[0];
        let guest = call.argument()[1];
        let size = call.argument()[2];

        realm::registry::get(vm)
            .ok_or("Not exist VM")?
            .lock()
            .page_table
            .lock()
            .unset_pages(GuestPhysAddr::from(guest), size);

        //TODO change GPT to nonsecure
        //TODO zeroize memory

        call.reply(rmi::RET_SUCCESS)?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMSetReg, |call| {
        let vm = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        let value = call.argument()[3];
        debug!(
            "received VMSetReg Reg[{}]={:#X} to VCPU {} on VM {}",
            register, value, vcpu, vm
        );
        match register {
            0..=30 => {
                realm::registry::get(vm)
                    .ok_or("Not exist VM")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .gp_regs[register] = value as u64;
                call.reply(rmi::RET_SUCCESS)
            }
            31 => {
                realm::registry::get(vm)
                    .ok_or("Not exist VM")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .elr = value as u64;
                call.reply(rmi::RET_SUCCESS)
            }
            32 => {
                realm::registry::get(vm)
                    .ok_or("Not exist VM")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .spsr = value as u64;
                call.reply(rmi::RET_SUCCESS)
            }
            _ => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMGetReg, |call| {
        let vm = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        debug!(
            "received VMGetReg Reg[{}] of VCPU {} on VM {}",
            register, vcpu, vm
        );
        match register {
            0..=30 => {
                let value = realm::registry::get(vm)
                    .ok_or("Not exist VM")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .gp_regs[register];
                call.reply(rmi::RET_SUCCESS)
                    .or(Err("RMM: failed to reply."))?;
                call.reply(value as usize)
            }
            31 => {
                let value = realm::registry::get(vm)
                    .ok_or("Not exist VM")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .elr;
                call.reply(rmi::RET_SUCCESS)?;
                call.reply(value as usize)
            }
            _ => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });
}
