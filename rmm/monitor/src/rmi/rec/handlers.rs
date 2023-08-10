use super::params::Params;
use super::run::{Run, REC_ENTRY_FLAG_TRAP_WFE, REC_ENTRY_FLAG_TRAP_WFI};
use super::Rec;
use crate::event::{realmexit, Context, Mainloop, RsiHandle};
use crate::listen;
use crate::rmi::realm::Rd;
use crate::{rmi, rsi};

use crate::host::pointer::Pointer as HostPointer;
use crate::host::pointer::PointerMut as HostPointerMut;
use crate::rmi::error::Error;
use crate::rmm::granule;
use crate::rmm::granule::GranuleState;

use core::mem::ManuallyDrop;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REC_CREATE, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        let rd = unsafe { Rd::into(arg[1]) };

        granule::set_granule(arg[0], GranuleState::Rec)?;
        mm.map(arg[0], true);

        let params = copy_from_host_or_ret!(Params, arg[2], mm);
        trace!("{:?}", params);

        match rmi.create_vcpu(rd.id()) {
            Ok(vcpuid) => {
                ret[1] = vcpuid;
                let _ =
                    unsafe { Rec::new(arg[0], vcpuid, ManuallyDrop::<&mut Rd>::into_inner(rd)) };
            }
            Err(_) => return Err(Error::RmiErrorInput),
        }

        let rec = unsafe { Rec::into(arg[0]) };
        let rd = unsafe { Rd::into(arg[1]) };
        for (idx, gpr) in params.gprs.iter().enumerate() {
            if rmi.set_reg(rd.id(), rec.id(), idx, *gpr as usize).is_err() {
                return Err(Error::RmiErrorInput);
            }
        }
        if rmi
            .set_reg(rd.id(), rec.id(), 31, params.pc as usize)
            .is_err()
        {
            return Err(Error::RmiErrorInput);
        }
        Ok(())
    });

    listen!(mainloop, rmi::REC_DESTROY, |arg, _ret, rmm| {
        granule::set_granule(arg[0], GranuleState::Delegated).map_err(|e| {
            rmm.mm.unmap(arg[0]);
            e
        })?;
        rmm.mm.unmap(arg[0]);
        Ok(())
    });

    listen!(mainloop, rmi::REC_ENTER, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mut rec = unsafe { Rec::into(arg[0]) };
        let run_pa = arg[1];
        let mut run = copy_from_host_or_ret!(Run, run_pa, rmm.mm);
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
        rmi.receive_gic_state_from_host(rec.rd.id(), rec.id(), &run)?;
        rmi.emulate_mmio(rec.rd.id(), rec.id(), &run)?;

        let ripas = rec.ripas_addr();
        if ripas > 0 {
            rmi.set_reg(rec.rd.id(), rec.id(), 0, 0)?;
            rmi.set_reg(rec.rd.id(), rec.id(), 1, ripas)?;
            rec.set_ripas(0, 0, 0, 0);
        }

        let wfx_flag = unsafe { run.entry_flags() };
        if wfx_flag & (REC_ENTRY_FLAG_TRAP_WFI | REC_ENTRY_FLAG_TRAP_WFE) != 0 {
            warn!("ISLET does not support re-configuring the WFI(E) trap");
            warn!("TWI(E) in HCR_EL2 is currently fixed to 'no trap'");
        }

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
                            if rsi.dispatch(&mut rsi_ctx, rmm, rec_ref, &mut run)
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
                        let _ = rmi.send_mmio_write(rec.rd.id(), rec.id(), &mut run);
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
        rmi.send_gic_state_to_host(rec.rd.id(), rec.id(), &mut run)?;
        rmi.send_timer_state_to_host(rec.rd.id(), rec.id(), &mut run)?;

        // NOTICE: do not modify `run` after copy_to_host_or_ret!(). it won't have any effect.
        copy_to_host_or_ret!(Run, &run, run_pa, rmm.mm);
        Ok(())
    });
}
