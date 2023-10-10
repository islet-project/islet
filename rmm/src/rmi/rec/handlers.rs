use super::mpidr::MPIDR;
use super::params::Params;
use super::run::{Run, REC_ENTRY_FLAG_TRAP_WFE, REC_ENTRY_FLAG_TRAP_WFI};
use super::vtcr::{activate_stage2_mmu, prepare_vtcr};
use super::Rec;
use crate::event::{realmexit, Context, Mainloop, RsiHandle};
use crate::granule::{set_granule, set_granule_with_parent, GranuleState};
use crate::host::pointer::Pointer as HostPointer;
use crate::host::pointer::PointerMut as HostPointerMut;
use crate::listen;
use crate::rmi::error::Error;
use crate::rmi::realm::{rd::State, Rd};
use crate::{get_granule, get_granule_if};
use crate::{rmi, rsi};

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REC_CREATE, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let rec = arg[0];
        let rd = arg[1];
        let params_ptr = arg[2];
        let owner = rd;

        if rec == rd {
            return Err(Error::RmiErrorInput);
        }

        let params = copy_from_host_or_ret!(Params, params_ptr);
        params.validate_aux(rec, rd, params_ptr)?;

        let rec_index = MPIDR::from(params.mpidr).index();
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let rd = rd_granule.content_mut::<Rd>();
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm);
        }

        if rec_index != rd.rec_index() {
            return Err(Error::RmiErrorInput);
        }

        // set Rec_state and grab the lock for Rec granule
        let mut rec_granule = get_granule_if!(rec, GranuleState::Delegated)?;
        rmm.page_table.map(rec, true);
        let rec = rec_granule.content_mut::<Rec>();

        match rmi.create_vcpu(rd.id()) {
            Ok(vcpuid) => {
                ret[1] = vcpuid;
                rec.init(owner, rd.id(), rd.state(), vcpuid);
            }
            Err(_) => return Err(Error::RmiErrorInput),
        }

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
        rec.set_vtcr(prepare_vtcr(rd)?);

        rd.inc_rec_index();
        set_granule_with_parent(rd_granule.clone(), &mut rec_granule, GranuleState::Rec)
    });

    listen!(mainloop, rmi::REC_DESTROY, |arg, _ret, rmm| {
        let mut rec_granule = get_granule_if!(arg[0], GranuleState::Rec)?;

        set_granule(&mut rec_granule, GranuleState::Delegated).map_err(|e| {
            rmm.page_table.unmap(arg[0]);
            e
        })?;
        rmm.page_table.unmap(arg[0]);
        Ok(())
    });

    listen!(mainloop, rmi::REC_ENTER, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let run_pa = arg[1];

        // grab the lock for Rec
        let mut rec_granule = get_granule_if!(arg[0], GranuleState::Rec)?;
        let mut rec = rec_granule.content_mut::<Rec>();

        {
            let rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
            let rd = rd.content::<Rd>();
            if !rd.at_state(State::Active) {
                return Err(Error::RmiErrorRealm);
            }
        }

        // read Run
        let mut run = copy_from_host_or_ret!(Run, run_pa);
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

        let ripas = rec.ripas_addr() as usize;
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

        activate_stage2_mmu(rec);

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
                            rsi_ctx.resize_ret(ret_num);

                            // set default value
                            if rsi.dispatch(&mut rsi_ctx, rmm, &mut rec, &mut run)
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
        copy_to_host_or_ret!(Run, &run, run_pa);
        Ok(())
    });
}
