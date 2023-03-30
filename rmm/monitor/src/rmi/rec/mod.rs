pub(crate) mod params;

use self::params::Params;
use super::gpt::mark_realm;
use crate::event::Mainloop;
use crate::listen;
use crate::{rmi, rsi};

extern crate alloc;

// TODO: Bind rd with realm & rec
pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REC_CREATE, |ctx, rmi, smc| {
        ctx.ret[0] = rmi::RET_FAIL;

        if rmi.create_vcpu(0).is_err() {
            return;
        }

        let addr = ctx.arg[2];
        if mark_realm(smc, addr)[0] != 0 {
            return;
        }

        let param = unsafe { Params::parse(addr) };
        trace!("{:?}", param);
        if rmi.set_reg(0, 0, 31, param.pc() as usize).is_err() {
            return;
        }

        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REC_ENTER, |ctx, rmi, _| {
        match rmi.run(0, 0, 0) {
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
