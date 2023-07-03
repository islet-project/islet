mod params;
pub mod run;

use self::params::Params;
use self::run::Run;
use super::realm::Rd;
use crate::event::{realmexit, Context, Mainloop, RsiHandle};
use crate::listen;
use crate::{rmi, rsi};

use crate::rmm::granule;
use crate::rmm::granule::GranuleState;

use core::mem::ManuallyDrop;

extern crate alloc;

pub struct Rec {
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
    listen!(mainloop, rmi::REC_CREATE, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        let rd = unsafe { Rd::into(arg[1]) };
        let params_ptr = arg[2];

        if granule::set_granule(arg[0], GranuleState::Rec, mm) != granule::RET_SUCCESS {
            ret[0] = rmi::ERROR_INPUT;
            return;
        }
        mm.map(params_ptr, false);
        ret[0] = rmi::RET_FAIL;

        match rmi.create_vcpu(rd.id()) {
            Ok(vcpuid) => {
                ret[1] = vcpuid;
                let _ =
                    unsafe { Rec::new(arg[0], vcpuid, ManuallyDrop::<&mut Rd>::into_inner(rd)) };
            }
            Err(_) => return,
        }

        let params = unsafe { Params::parse(params_ptr) };
        trace!("{:?}", params);
        let rec = unsafe { Rec::into(arg[0]) };
        let rd = unsafe { Rd::into(arg[1]) };
        for (idx, gpr) in params.gprs().iter().enumerate() {
            if rmi.set_reg(rd.id(), rec.id(), idx, *gpr as usize).is_err() {
                mm.unmap(params_ptr);
                return;
            }
        }
        if rmi
            .set_reg(rd.id(), rec.id(), 31, params.pc() as usize)
            .is_err()
        {
            mm.unmap(params_ptr);
            return;
        }
        mm.unmap(params_ptr);
        ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REC_DESTROY, |arg, ret, rmm| {
        if granule::set_granule(arg[0], GranuleState::Delegated, rmm.mm) != granule::RET_SUCCESS {
            ret[0] = rmi::ERROR_INPUT;
            return;
        }
        ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REC_ENTER, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let rec = unsafe { Rec::into(arg[0]) };
        let run_ptr = arg[1];
        rmm.mm.map(run_ptr, false);

        let run = unsafe { Run::parse_mut(run_ptr) };
        trace!("{:?}", run);

        unsafe {
            // TODO: copy rec::entry gprs to host_call gprs
            let ipa: u64 = 0x800088e00000;
            if run.entry_gpr0() == ipa {
                // TODO: Get ipa from rec->regs[1] and map to pa
                let pa: usize = 0x88b0_6000;
                let host_call = rsi::hostcall::HostCall::parse_mut(pa);
                host_call.set_gpr0(ipa);
            }
        }
        let _ = rmi.receive_gic_state_from_host(rec.rd.id(), rec.id(), run);

        let mut ret_ns;
        loop {
            ret_ns = true;
            match rmi.run(rec.rd.id(), rec.id(), 0) {
                Ok(val) => match val[0] {
                    realmexit::RSI => {
                        trace!("REC_ENTER ret: {:#X?}", val);
                        let rsi = &rmm.rsi;
                        let cmd = val[1];

                        rsi::constraint::validate(cmd, |_, ret_num| {
                            let mut rsi_ctx = Context::new(cmd);
                            let rec_ref =
                                unsafe { ManuallyDrop::<&mut Rec>::into_inner(Rec::into(arg[0])) };
                            rsi_ctx.resize_ret(ret_num);

                            // set default value
                            if rsi.dispatch(&mut rsi_ctx, rmm, rec_ref, run)
                                == RsiHandle::RET_SUCCESS
                            {
                                if rsi_ctx.ret_slice()[0] == rmi::SUCCESS_REC_ENTER {
                                    ret_ns = false;
                                }
                                ret[0] = rsi_ctx.ret_slice()[0];
                            } else {
                                ret_ns = false;
                            }
                        });
                    }
                    realmexit::SYNC => unsafe {
                        run.set_exit_reason(rmi::EXIT_SYNC);
                        run.set_esr(val[1] as u64);
                        run.set_hpfar(val[2] as u64);
                        run.set_far(val[3] as u64);
                        ret[0] = rmi::SUCCESS;
                    },
                    realmexit::IRQ => unsafe {
                        run.set_exit_reason(rmi::EXIT_IRQ);
                        run.set_esr(val[1] as u64);
                        run.set_hpfar(val[2] as u64);
                        run.set_far(val[3] as u64);
                        ret[0] = rmi::SUCCESS;
                    },
                    _ => ret[0] = rmi::SUCCESS,
                },
                Err(_) => ret[0] = rmi::ERROR_REC,
            };
            if ret_ns == true {
                break;
            }
        }
        let _ = rmi.send_gic_state_to_host(rec.rd.id(), rec.id(), run);
        rmm.mm.unmap(run_ptr);
    });
}
