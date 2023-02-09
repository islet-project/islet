use crate::error::{Error, ErrorKind};
use crate::listen;
use crate::mainloop::Mainloop;
use crate::realm::vm;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::VMCreate, |call| {
        info!("received VMCreate");

        let id = vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .create()?;
        call.reply(rmi::RET_SUCCESS)?;
        call.reply(id)?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VCPUCreate, |call| {
        let id = call.argument()[0];
        // let vcpu = call.argument()[1];
        debug!("received VCPUCreate for VM {}", id);

        let ret = vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .create_vcpu(id);
        match ret {
            Ok(vcpuid) => {
                call.reply(rmi::RET_SUCCESS)?;
                call.reply(vcpuid)
            }
            Err(_) => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMDestroy, |call| {
        let id = call.argument()[0];
        info!("received VMDestroy VM {}", id);

        vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .remove(id)?;
        call.reply(rmi::RET_SUCCESS)?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMRun, |call| {
        let id = call.argument()[0];
        let vcpu = call.argument()[1];
        let incr_pc = call.argument()[2];
        debug!(
            "received VMRun VCPU {} on VM {} PC_INCR {}",
            vcpu, id, incr_pc
        );
        let ret = vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .run(id, vcpu, incr_pc);
        match ret {
            Ok(val) => match val[0] {
                rmi::RET_EXCEPTION_TRAP | rmi::RET_EXCEPTION_IRQ => {
                    call.reply(val[0]).or(Err("RMM failed to reply."))?;
                    call.reply(val[1])?;
                    call.reply(val[2])?;
                    call.reply(val[3])
                }
                _ => call.reply(rmi::RET_SUCCESS),
            },
            Err(_) => Err(Error::new(ErrorKind::Unsupported)),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMMapMemory, |call| {
        let id = call.argument()[0];
        let guest = call.argument()[1];
        let phys = call.argument()[2];
        let size = call.argument()[3];
        // prot: bits[0] : writable, bits[1] : fault on exec, bits[2] : device
        let prot = call.argument()[4]; // bits[3]
        debug!(
            "received MapMemory to VM {} {:#X} -> {:#X} size:{:#X} prot:{:#X}",
            id, guest, phys, size, prot
        );
        let ret = vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .map(id, guest, phys, size, prot);
        match ret {
            Ok(_) => call.reply(rmi::RET_SUCCESS)?,
            Err(_) => call.reply(rmi::RET_FAIL)?,
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMUnmapMemory, |call| {
        let id = call.argument()[0];
        let guest = call.argument()[1];
        let size = call.argument()[2];
        debug!(
            "received UnmapMemory to VM {} {:#X}, size:{:#X}",
            id, guest, size
        );
        let ret = vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .unmap(id, guest, size);
        match ret {
            Ok(_) => call.reply(rmi::RET_SUCCESS)?,
            Err(_) => call.reply(rmi::RET_FAIL)?,
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMSetReg, |call| {
        let id = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        let value = call.argument()[3];
        debug!(
            "received VMSetReg Reg[{}]={:#X} to VCPU {} on VM {}",
            register, value, vcpu, id
        );
        let ret = vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .set_reg(id, vcpu, register, value);
        match ret {
            Ok(_) => call.reply(rmi::RET_SUCCESS)?,
            Err(_) => call.reply(rmi::RET_FAIL)?,
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMGetReg, |call| {
        let id = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        debug!(
            "received VMGetReg Reg[{}] of VCPU {} on VM {}",
            register, vcpu, id
        );
        let ret = vm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .get_reg(id, vcpu, register);
        match ret {
            Ok(value) => {
                call.reply(rmi::RET_SUCCESS)?;
                call.reply(value)?;
            }
            Err(_) => call.reply(rmi::RET_FAIL)?,
        }
        Ok(())
    });
}
