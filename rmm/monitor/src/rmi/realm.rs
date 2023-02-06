use crate::mainloop::Mainloop;
use crate::error::{Error, ErrorKind};
use crate::listen;
use crate::rmi;

extern crate alloc;

pub type VMManager = &'static dyn crate::realm::vm::VMControl;
static mut VMM: Option<VMManager> = None;

#[allow(unused_must_use)]
pub fn set_instance(vm: VMManager) {
    unsafe {
        if VMM.is_none() {
            VMM = Some(vm);
        }
    };
}

pub fn instance() -> Option<VMManager> {
    unsafe { VMM }
}

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::VMCreate, |call| {
        info!("received VMCreate");
        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();
        let id = vmm.create().unwrap();
        call.reply(rmi::RET_SUCCESS)?;
        call.reply(id)?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VCPUCreate, |call| {
        let id = call.argument()[0];
        // let vcpu = call.argument()[1];
        debug!("received VCPUCreate for VM {}", id);

        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();

        match vmm.create_vcpu(id) {
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
        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();

        info!("received VMDestroy VM {}", id);
        vmm.remove(id)?;
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
        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();
        match vmm.run(id, vcpu, incr_pc) {
            Ok(val) => match val[0] {
                rmi::RET_EXCEPTION_TRAP | rmi::RET_EXCEPTION_IRQ => {
                    call.reply(val[0]).or(Err("RMM failed to reply."))?;
                    call.reply(val[1])?;
                    call.reply(val[2])?;
                    call.reply(val[3])
                }
                _ => call.reply(rmi::RET_SUCCESS),
            },
            _ => Err(Error::new(ErrorKind::Unsupported)),
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
        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();
        match vmm.map(id, guest, phys, size, prot) {
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
            "received UnmapMemory to VM {} {:#X}, size:{:#X}", id, guest, size);

        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();
        match vmm.unmap(id, guest, size) {
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

        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();
        match vmm.set_reg(id, vcpu, register, value) {
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

        let vmm = instance().ok_or(Error::new(ErrorKind::Unsupported)).unwrap();
        match vmm.get_reg(id, vcpu, register) {
            Ok(value) => {
                call.reply(rmi::RET_SUCCESS)?;
                call.reply(value)?;
            },
            Err(_) => call.reply(rmi::RET_FAIL)?,
        }
        Ok(())
    });
}