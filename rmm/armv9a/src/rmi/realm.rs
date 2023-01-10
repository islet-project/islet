use crate::realm::vm::{VMControl, VMManager, VMMemory, VMRegister};
use crate::rmi::Receiver;
use monitor::{listen, mainloop::Mainloop, rmi};

pub fn set_event_handler(mainloop: &mut Mainloop<Receiver>) {
    listen!(mainloop, rmi::Code::VMCreate, |call| {
        info!("received VMCreate");
        let mut vm = VMManager { id: 0 };
        vm.new();
        info!("create VM {}", vm.id);
        call.reply(rmi::RET_SUCCESS)?;
        call.reply(vm.id)?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VCPUCreate, |call| {
        let vm = VMManager {
            id: call.argument()[0],
        };
        // let vcpu = call.argument()[1];
        debug!("received VCPUCreate for VM {}", vm.id);
        match vm.get().ok_or("Not exist VM")?.lock().create_vcpu() {
            Ok(vcpuid) => {
                call.reply(rmi::RET_SUCCESS)?;
                call.reply(vcpuid)
            }
            Err(_) => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMDestroy, |call| {
        let vm = VMManager {
            id: call.argument()[0],
        };
        info!("received VMDestroy VM {}", vm.id);
        match vm.remove() {
            Ok(_) => call.reply(rmi::RET_SUCCESS),
            Err(_) => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMRun, |call| {
        let vm = VMManager {
            id: call.argument()[0],
        };
        let vcpu = call.argument()[1];
        let incr_pc = call.argument()[2];
        debug!(
            "received VMRun VCPU {} on VM {} PC_INCR {}",
            vcpu, vm.id, incr_pc
        );

        let ret = vm.run(vcpu, incr_pc);
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
            Err(_) => call.reply(rmi::RET_FAIL),
        }?;
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMMapMemory, |call| {
        let vm = VMManager {
            id: call.argument()[0],
        };
        let guest = call.argument()[1];
        let phys = call.argument()[2];
        let size = call.argument()[3];
        // prot: bits[0] : writable, bits[1] : fault on exec, bits[2] : device
        let prot = call.argument()[4]; // bits[3]
        debug!(
            "received MapMemory to VM {} {:#X} -> {:#X} size:{:#X} prot:{:#X}",
            vm.id, guest, phys, size, prot
        );

        match vm.map(guest, phys, size, prot) {
            Err(_) => call.reply(rmi::RET_FAIL)?,
            _ => call.reply(rmi::RET_SUCCESS)?,
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMUnmapMemory, |call| {
        let vm = VMManager {
            id: call.argument()[0],
        };
        let guest = call.argument()[1];
        let size = call.argument()[2];

        match vm.unmap(guest, size) {
            Err(_) => call.reply(rmi::RET_FAIL)?,
            _ => call.reply(rmi::RET_SUCCESS)?,
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMSetReg, |call| {
        let vm = VMManager {
            id: call.argument()[0],
        };
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        let value = call.argument()[3];
        debug!(
            "received VMSetReg Reg[{}]={:#X} to VCPU {} on VM {}",
            register, value, vcpu, vm.id
        );

        match vm.set_reg(vcpu, register, value) {
            Err(_) => call.reply(rmi::RET_FAIL)?,
            _ => call.reply(rmi::RET_SUCCESS)?,
        }
        Ok(())
    });

    listen!(mainloop, rmi::Code::VMGetReg, |call| {
        let vm = VMManager {
            id: call.argument()[0],
        };
        let vcpu = call.argument()[1];
        let register = call.argument()[2];
        debug!(
            "received VMGetReg Reg[{}] of VCPU {} on VM {}",
            register, vcpu, vm.id
        );

        match vm.get_reg(vcpu, register) {
            Err(_) => call.reply(rmi::RET_FAIL)?,
            _ => call.reply(rmi::RET_SUCCESS)?,
        }
        Ok(())
    });
}
