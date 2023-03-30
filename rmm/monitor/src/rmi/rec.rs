use crate::event::Mainloop;
use crate::listen;
use crate::{rmi, rsi};

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    // related with:
    //   - VCPU_CREATE
    listen!(mainloop, rmi::REC_CREATE, |ctx, rmi, _| {
        let ret = rmi.create_vcpu(0);
        match ret {
            Ok(vcpuid) => {
                ctx.ret[0] = rmi::SUCCESS;
                ctx.ret[1] = vcpuid; // TODO: Binding to RD
            }
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    // related with:
    //   - REALM_RUN
    listen!(mainloop, rmi::REC_ENTER, |ctx, rmi, _| {
        let _ = rmi.set_reg(0, 0, 31, 0x88b00000);
        let ret = rmi.run(0, 0, 0);
        match ret {
            Ok(val) => match val[0] {
                rsi::HOST_CALL => {
                    trace!("HOST_CALL: {:#X?}", val);
                    // This point means that realm is executed
                    // TODO: Parse rsi_host_call data structure
                    ctx.ret[0] = rmi::ERROR_REC;
                }
                rmi::RET_EXCEPTION_TRAP | rmi::RET_EXCEPTION_IRQ => {
                    ctx.ret = [val[0], val[1], val[2], val[3], 0, 0, 0, 0];
                }
                _ => ctx.ret[0] = rmi::SUCCESS,
            },
            Err(_) => ctx.ret[0] = rmi::ERROR_REC,
        };
    });
}
