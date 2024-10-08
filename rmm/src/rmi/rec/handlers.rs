use super::mpidr::MPIDR;
use super::params::Params;
use super::run::{Run, REC_ENTRY_FLAG_TRAP_WFE, REC_ENTRY_FLAG_TRAP_WFI};
use super::vtcr::{activate_stage2_mmu, prepare_vtcr};
use crate::event::Mainloop;
#[cfg(feature = "gst_page_table")]
use crate::granule::{set_granule, set_granule_with_parent, GranuleState};
#[cfg(not(feature = "gst_page_table"))]
use crate::granule::{set_granule, GranuleState};
use crate::host;
use crate::listen;
use crate::measurement::HashContext;
use crate::realm::context::{set_reg, RegOffset};
use crate::realm::rd::{Rd, State};
use crate::rec::Rec;
use crate::rec::State as RecState;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmi::rec::exit::handle_realm_exit;
use crate::rsi::do_host_call;
use crate::rsi::psci::complete_psci;
use crate::{get_granule, get_granule_if};

use aarch64_cpu::registers::*;
use armv9a::bits_in_reg;

extern crate alloc;

fn prepare_args(rd: &mut Rd, mpidr: u64) -> Result<(usize, u64, u64), Error> {
    let page_table = rd.s2_table().lock().get_base_address() as u64;
    let vttbr = bits_in_reg(
        VTTBR_EL2::VMID.mask << VTTBR_EL2::VMID.shift,
        rd.id() as u64,
    ) | bits_in_reg(
        VTTBR_EL2::BADDR.mask << VTTBR_EL2::BADDR.shift,
        page_table >> 1,
    );
    let vmpidr = mpidr | (MPIDR_EL1::RES1.mask << MPIDR_EL1::RES1.shift);
    let vcpuid = rd.vcpu_index;
    rd.vcpu_index += 1;
    Ok((vcpuid, vttbr, vmpidr))
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REC_CREATE, |arg, ret, rmm| {
        let rd = arg[0];
        let rec = arg[1];
        let params_ptr = arg[2];
        let owner = rd;

        if rec == rd {
            return Err(Error::RmiErrorInput);
        }

        let params = host::copy_from::<Params>(params_ptr).ok_or(Error::RmiErrorInput)?;
        params.verify_compliance(rec, rd, params_ptr)?;

        let rec_index = MPIDR::from(params.mpidr).index();
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }

        if rec_index != rd.rec_index() {
            return Err(Error::RmiErrorInput);
        }
        // set Rec_state and grab the lock for Rec granule
        let mut rec_granule = get_granule_if!(rec, GranuleState::Delegated)?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(rec, true);
        let mut rec = rec_granule.new_uninit_with::<Rec<'_>>(Rec::new())?;
        match prepare_args(&mut rd, params.mpidr) {
            Ok((vcpuid, vttbr, vmpidr)) => {
                ret[1] = vcpuid;
                rec.init(owner, vcpuid, params.flags, params.aux, vttbr, vmpidr)?;
            }
            Err(_) => return Err(Error::RmiErrorInput),
        }

        for (idx, gpr) in params.gprs.iter().enumerate() {
            if set_reg(&mut rec, idx, *gpr as usize).is_err() {
                return Err(Error::RmiErrorInput);
            }
        }
        if set_reg(&mut rec, RegOffset::PC, params.pc as usize).is_err() {
            return Err(Error::RmiErrorInput);
        }
        rec.set_vtcr(prepare_vtcr(&rd)?);

        rd.inc_rec_index();
        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(&mut rd)?.measure_rec_params(&params)?;

        for i in 0..rmi::MAX_REC_AUX_GRANULES {
            let rec_aux = rec.aux(i) as usize;
            let mut rec_aux_granule = get_granule_if!(rec_aux, GranuleState::Delegated)?;
            set_granule(&mut rec_aux_granule, GranuleState::RecAux)?;
        }

        #[cfg(feature = "gst_page_table")]
        return set_granule_with_parent(rd_granule.clone(), &mut rec_granule, GranuleState::Rec);
        #[cfg(not(feature = "gst_page_table"))]
        return set_granule(&mut rec_granule, GranuleState::Rec);
    });

    listen!(mainloop, rmi::REC_DESTROY, |arg, _ret, rmm| {
        let mut rec_granule = get_granule_if!(arg[0], GranuleState::Rec)?;

        set_granule(&mut rec_granule, GranuleState::Delegated).map_err(|e| {
            #[cfg(not(kani))]
            // `page_table` is currently not reachable in model checking harnesses
            rmm.page_table.unmap(arg[0]);
            e
        })?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.unmap(arg[0]);
        Ok(())
    });

    listen!(mainloop, rmi::REC_ENTER, |arg, ret, rmm| {
        let run_pa = arg[1];

        // grab the lock for Rec
        let mut rec_granule = get_granule_if!(arg[0], GranuleState::Rec)?;
        let mut rec = rec_granule.content_mut::<Rec<'_>>()?;

        // read Run
        let mut run = host::copy_from::<Run>(run_pa).ok_or(Error::RmiErrorInput)?;
        run.verify_compliance()?;
        trace!("{:?}", run);

        let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        match rd.state() {
            State::Active => {}
            State::New => {
                return Err(Error::RmiErrorRealm(0));
            }
            State::SystemOff => {
                return Err(Error::RmiErrorRealm(1));
            }
            _ => {
                panic!("Unexpected realm state");
            }
        }
        // XXX: we explicitly release Rd's lock here to avoid a deadlock
        core::mem::drop(rd_granule);

        // runnable oder is lower
        if !rec.runnable() {
            return Err(Error::RmiErrorRec);
        }

        if let RecState::Running = rec.get_state() {
            error!("Rec is already running: {:?}", *rec);
            return Err(Error::RmiErrorRec);
        }

        if rec.psci_pending() {
            return Err(Error::RmiErrorRec);
        }

        if !crate::gic::validate_state(&run) {
            return Err(Error::RmiErrorRec);
        }

        if rec.host_call_pending() {
            // The below should be called without holding rd's lock
            do_host_call(arg, ret, rmm, &mut rec, &mut run)?;
        }

        crate::gic::receive_state_from_host(&mut rec, &run)?;
        crate::mmio::emulate_mmio(&mut rec, &run)?;

        crate::rsi::ripas::complete_ripas(&mut rec, &run)?;

        let wfx_flag = run.entry_flags();
        if wfx_flag & (REC_ENTRY_FLAG_TRAP_WFI | REC_ENTRY_FLAG_TRAP_WFE) != 0 {
            warn!("Islet does not support re-configuring the WFI(E) trap");
            warn!("TWI(E) in HCR_EL2 is currently fixed to 'no trap'");
        }

        activate_stage2_mmu(&rec);

        let mut ret_ns;
        loop {
            ret_ns = true;
            run.set_imm(0);

            let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
            let rd = rd_granule.content::<Rd>()?;
            rec.set_state(RecState::Running);
            crate::rec::run_prepare(&rd, rec.vcpuid(), &mut rec, 0)?;
            // XXX: we explicitly release Rd's lock here, because RSI calls
            //      would acquire the same lock again (deadlock).
            core::mem::drop(rd_granule);
            match crate::rec::run() {
                Ok(realm_exit_res) => {
                    (ret_ns, ret[0]) = handle_realm_exit(realm_exit_res, rmm, &mut rec, &mut run)?
                }
                Err(_) => ret[0] = rmi::ERROR_REC,
            }
            rec.set_state(RecState::Ready);

            if ret_ns {
                break;
            }
        }
        crate::gic::send_state_to_host(&rec, &mut run)?;
        crate::realm::timer::send_state_to_host(&rec, &mut run)?;

        // NOTICE: do not modify `run` after copy_to_ptr(). it won't have any effect.
        host::copy_to_ptr::<Run>(&run, run_pa).ok_or(Error::RmiErrorInput)
    });

    listen!(mainloop, rmi::PSCI_COMPLETE, |arg, _ret, _rmm| {
        let caller_pa = arg[0];
        let target_pa = arg[1];

        if caller_pa == target_pa {
            return Err(Error::RmiErrorInput);
        }
        let mut caller_granule = get_granule_if!(caller_pa, GranuleState::Rec)?;
        let mut caller = caller_granule.content_mut::<Rec<'_>>()?;

        let mut target_granule = get_granule_if!(target_pa, GranuleState::Rec)?;
        let mut target = target_granule.content_mut::<Rec<'_>>()?;

        let status = arg[2];

        if !caller.psci_pending() {
            return Err(Error::RmiErrorInput);
        }

        if caller.realmid()? != target.realmid()? {
            return Err(Error::RmiErrorInput);
        }

        complete_psci(&mut caller, &mut target, status)
    });
}
