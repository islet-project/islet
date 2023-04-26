mod params;
mod run;

use self::params::Params;
use self::run::Run;
use super::gpt::{mark_ns, mark_realm};
use super::realm::Rd;
use crate::event::Mainloop;
use crate::listen;
use crate::{rmi, rsi};

use core::mem::ManuallyDrop;

extern crate alloc;

struct Rec {
    pub rd: &'static Rd,
    vcpuid: usize,
}

impl Rec {
    pub unsafe fn new(
        rec_addr: usize,
        vcpuid: usize,
        rd: &'static Rd,
    ) -> ManuallyDrop<&'static mut Rec> {
        let rec: &mut Rec = &mut *(rec_addr as *mut Rec);
        rec.vcpuid = vcpuid;
        rec.rd = rd;
        ManuallyDrop::new(rec)
    }

    pub unsafe fn into(rec_addr: usize) -> ManuallyDrop<&'static mut Rec> {
        let rec: &mut Rec = &mut *(rec_addr as *mut Rec);
        ManuallyDrop::new(rec)
    }

    pub fn id(&self) -> usize {
        self.vcpuid
    }
}

impl Drop for Rec {
    fn drop(&mut self) {}
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REC_CREATE, |ctx, rmm| {
        let mm = rmm.mm;
        let rmi = rmm.rmi;
        let smc = rmm.smc;
        let _ = mm.map([ctx.arg[0], ctx.arg[2], 0, 0]);
        let rd = unsafe { Rd::into(ctx.arg[1]) };
        let params_ptr = ctx.arg[2];
        ctx.ret[0] = rmi::RET_FAIL;

        match rmi.create_vcpu(rd.id()) {
            Ok(vcpuid) => {
                ctx.ret[1] = vcpuid;
                let _ = unsafe {
                    Rec::new(ctx.arg[0], vcpuid, ManuallyDrop::<&mut Rd>::into_inner(rd))
                };
            }
            Err(_) => return,
        }

        if mark_realm(smc, params_ptr)[0] != 0 {
            return;
        }

        let params = unsafe { Params::parse(params_ptr) };
        trace!("{:?}", params);
        let rec = unsafe { Rec::into(ctx.arg[0]) };
        let rd = unsafe { Rd::into(ctx.arg[1]) };
        if rmi
            .set_reg(rd.id(), rec.id(), 31, params.pc() as usize)
            .is_err()
        {
            return;
        }

        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REC_DESTROY, |ctx, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REC_ENTER, |ctx, rmm| {
        let mm = rmm.mm;
        let rmi = rmm.rmi;
        let smc = rmm.smc;
        let _rec = unsafe { Rec::into(ctx.arg[0]) };
        let run_ptr = ctx.arg[1];
        let _ = mm.map([run_ptr, 0, 0, 0]);
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
                let pa: usize = 0x88b0_6000;
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
                    let pa: usize = ipa;
                    unsafe {
                        let host_call = rsi::HostCall::parse(pa);
                        run.set_imm(host_call.imm());
                        run.set_exit_reason(rmi::EXIT_HOST_CALL);

                        trace!("HOST_CALL param: {:#X?}", host_call);
                    };
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
