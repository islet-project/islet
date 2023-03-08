use crate::error::{Error, ErrorKind};
use crate::listen;
use crate::mainloop::Mainloop;
use crate::realm;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::RealmCreate, |call| {
        info!("received RealmCreate");

        let id = realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .create()?;
        call.reply(&[rmi::RET_SUCCESS, id, 0, 0]);
        Ok(())
    });

    listen!(mainloop, rmi::Code::VCPUCreate, |call| {
        let id = call.argument()[0];
        // let vcpu = call.argument()[1];
        debug!("received VCPUCreate for VM {}", id);

        let ret = realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .create_vcpu(id);
        match ret {
            Ok(vcpuid) => call.reply(&[rmi::RET_SUCCESS, vcpuid, 0, 0]),
            Err(_) => call.reply(&[rmi::RET_FAIL, 0, 0, 0]),
        };
        Ok(())
    });

    listen!(mainloop, rmi::Code::RealmDestroy, |call| {
        let id = call.argument()[0];
        info!("received RealmDestroy Realm {}", id);

        realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .remove(id)?;
        call.reply(&[rmi::RET_SUCCESS, 0, 0, 0]);
        Ok(())
    });

    listen!(mainloop, rmi::Code::RealmRun, |call| {
        let id = call.argument()[0];
        let vcpu = call.argument()[1];
        let incr_pc = call.argument()[2];
        debug!(
            "received RealmRun VCPU {} on Realm {} PC_INCR {}",
            vcpu, id, incr_pc
        );
        let ret = realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .run(id, vcpu, incr_pc);
        match ret {
            Ok(val) => match val[0] {
                rmi::RET_EXCEPTION_TRAP | rmi::RET_EXCEPTION_IRQ => {
                    call.reply(&[val[0], val[1], val[2], val[3]])
                }
                _ => call.reply(&[rmi::RET_SUCCESS, 0, 0, 0]),
            },
            Err(_) => call.reply(&[rmi::RET_FAIL, 0, 0, 0]),
        };
        Ok(())
    });

    listen!(mainloop, rmi::Code::RealmMapMemory, |call| {
        let id = call.argument()[0];
        let guest = call.argument()[1];
        let phys = call.argument()[2];
        let size = call.argument()[3];
        // prot: bits[0] : writable, bits[1] : fault on exec, bits[2] : device
        let prot = call.argument()[4]; // bits[3]
        debug!(
            "received MapMemory to Realm {} {:#X} -> {:#X} size:{:#X} prot:{:#X}",
            id, guest, phys, size, prot
        );
        let ret = realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .map(id, guest, phys, size, prot);
        match ret {
            Ok(_) => call.reply(&[rmi::RET_SUCCESS, 0, 0, 0]),
            Err(_) => call.reply(&[rmi::RET_FAIL, 0, 0, 0]),
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::RealmUnmapMemory, |call| {
        let id = call.argument()[0];
        let guest = call.argument()[1];
        let size = call.argument()[2];
        debug!(
            "received UnmapMemory to Realm {} {:#X}, size:{:#X}",
            id, guest, size
        );
        let ret = realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .unmap(id, guest, size);
        match ret {
            Ok(_) => call.reply(&[rmi::RET_SUCCESS, 0, 0, 0]),
            Err(_) => call.reply(&[rmi::RET_FAIL, 0, 0, 0]),
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::RealmSetReg, |call| {
        let id = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        let value = call.argument()[3];
        debug!(
            "received RealmSetReg Reg[{}]={:#X} to VCPU {} on Realm {}",
            register, value, vcpu, id
        );
        let ret = realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .set_reg(id, vcpu, register, value);
        match ret {
            Ok(_) => call.reply(&[rmi::RET_SUCCESS, 0, 0, 0]),
            Err(_) => call.reply(&[rmi::RET_FAIL, 0, 0, 0]),
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::RealmGetReg, |call| {
        let id = call.argument()[0];
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        debug!(
            "received RealmGetReg Reg[{}] of VCPU {} on Realm {}",
            register, vcpu, id
        );
        let ret = realm::instance()
            .ok_or(Error::new(ErrorKind::Unsupported))?
            .get_reg(id, vcpu, register);
        match ret {
            Ok(val) => call.reply(&[rmi::RET_SUCCESS, val, 0, 0]),
            Err(_) => call.reply(&[rmi::RET_FAIL, 0, 0, 0]),
        }
        Ok(())
    });
}
