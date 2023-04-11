mod params;
mod run;

use self::params::Params;
use self::run::Run;
use super::gpt::{mark_ns, mark_realm};
use crate::event::Mainloop;
use crate::listen;
use crate::{rmi, rsi};

extern crate alloc;

// TODO: Bind rd with realm & rec
pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REC_CREATE, |ctx, rmi, smc| {
        let _rec = ctx.arg[0];
        let rd = ctx.arg[1];
        let params_ptr = ctx.arg[2];
        let realm_id = rd;
        ctx.ret[0] = rmi::RET_FAIL;

        match rmi.create_vcpu(realm_id) {
            Ok(vcpuid) => ctx.ret[1] = vcpuid,
            Err(_) => return,
        }

        if mark_realm(smc, params_ptr)[0] != 0 {
            return;
        }

        let params = unsafe { Params::parse(params_ptr) };
        trace!("{:?}", params);
        if rmi.set_reg(0, 0, 31, params.pc() as usize).is_err() {
            return;
        }

        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REC_DESTROY, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REC_ENTER, |ctx, rmi, smc| {
        let _rec = ctx.arg[0];
        let run_ptr = ctx.arg[1];
        if mark_realm(smc, run_ptr)[0] != 0 {
            return;
        }

        let run = unsafe { Run::parse_mut(run_ptr) };
        trace!("{:?}", run);

        unsafe {
            // TODO: copy rec::entry gprs to host_call gprs
            let ipa: u64 = 0x800088e00000;
            if run.entry_gpr0() == ipa {
                // TODO: Get ipa from rec->regs[1] and map to pa
                let pa: usize = 0x88a0_6000;
                let host_call = rsi::HostCall::parse_mut(pa);
                host_call.set_gpr0(ipa);
            }
        }

        // set smc ret(x0) to RSI_SUCCESS
        if rmi.set_reg(0, 0, 0, 0).is_err() {
            return;
        }

        match rmi.run(0, 0, 0) {
            Ok(val) => match val[0] {
                rsi::HOST_CALL => {
                    trace!("REC_ENTER ret: {:#X?}", val);
                    let ipa = val[1];
                    // TODO: ipa to pa
                    if ipa == 0x88b0_6000 {
                        let pa: usize = 0x88a0_6000;
                        unsafe {
                            let host_call = rsi::HostCall::parse(pa);
                            run.set_imm(host_call.imm());
                            run.set_exit_reason(rmi::EXIT_HOST_CALL);
                        };
                    }
                    ctx.ret[0] = rmi::SUCCESS;
                }
                rmi::RET_EXCEPTION_TRAP | rmi::RET_EXCEPTION_IRQ => {
                    ctx.ret = [val[0], val[1], val[2], val[3], 0, 0, 0, 0];
                }
                _ => ctx.ret[0] = rmi::SUCCESS,
            },
            Err(_) => ctx.ret[0] = rmi::ERROR_REC,
        };

        if mark_ns(smc, run_ptr)[0] != 0 {
            ctx.ret[0] = rmi::RET_FAIL;
            return;
        }
    });
}
