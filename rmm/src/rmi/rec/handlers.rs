use super::mpidr::MPIDR;
use super::params::Params;
use super::run::{Run, REC_ENTRY_FLAG_TRAP_WFE, REC_ENTRY_FLAG_TRAP_WFI};
use super::vtcr::{activate_stage2_mmu, prepare_vtcr};
use crate::event::RmiHandle;
#[cfg(feature = "gst_page_table")]
use crate::granule::{set_granule, set_granule_with_parent, GranuleState};
#[cfg(not(feature = "gst_page_table"))]
use crate::granule::{set_granule, GranuleState};
use crate::host;
use crate::listen;
use crate::measurement::HashContext;
use crate::realm::context::{set_reg, RegOffset};
use crate::realm::rd::{Rd, State};
use crate::rec::State as RecState;
use crate::rec::{Rec, RmmRecEmulatableAbort::NotEmulatableAbort};
use crate::rmi;
use crate::rmi::error::Error;
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

pub fn set_event_handler(rmi: &mut RmiHandle) {
    #[cfg(not(kani))]
    listen!(rmi, rmi::REC_CREATE, |arg, ret, rmm| {
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
            rmm.page_table.map(rec_aux, true);
            let mut rec_aux_granule = get_granule_if!(rec_aux, GranuleState::Delegated)?;
            set_granule(&mut rec_aux_granule, GranuleState::RecAux)?;
        }

        #[cfg(not(feature = "gst_page_table"))]
        rd_granule.inc_count();

        #[cfg(feature = "gst_page_table")]
        return set_granule_with_parent(rd_granule.clone(), &mut rec_granule, GranuleState::Rec);
        #[cfg(not(feature = "gst_page_table"))]
        return set_granule(&mut rec_granule, GranuleState::Rec);
    });

    #[cfg(any(not(kani), feature = "mc_rmi_rec_destroy"))]
    listen!(rmi, rmi::REC_DESTROY, |arg, _ret, rmm| {
        let mut rec_granule = get_granule_if!(arg[0], GranuleState::Rec)?;

        let rec = rec_granule.content::<Rec<'_>>()?;
        if rec.get_state() == RecState::Running {
            return Err(Error::RmiErrorRec);
        }

        #[cfg(not(kani))]
        for i in 0..rmi::MAX_REC_AUX_GRANULES {
            let rec_aux = rec.aux(i) as usize;
            let mut rec_aux_granule = get_granule_if!(rec_aux, GranuleState::RecAux)?;
            set_granule(&mut rec_aux_granule, GranuleState::Delegated)?;
            rmm.page_table.unmap(rec_aux);
        }
        #[cfg(kani)]
        {
            // XXX: we check only the first aux to reduce the overall
            //      verification time
            let rec_aux = rec.aux(0) as usize;
            // XXX: the below can be guaranteed by Rec's invariants instead
            kani::assume(crate::granule::validate_addr(rec_aux));
            let mut rec_aux_granule = get_granule!(rec_aux)?;
            set_granule(&mut rec_aux_granule, GranuleState::Delegated)?;
        }

        #[cfg(not(feature = "gst_page_table"))]
        {
            let rd = rec.owner()?;
            #[cfg(kani)]
            {
                // XXX: the below can be guaranteed by Rec's invariants instead
                kani::assume(crate::granule::validate_addr(rd));
                let rd_granule = get_granule!(rd)?;
                kani::assume(rd_granule.state() == GranuleState::RD);
            }
            let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
            rd_granule.dec_count();
        }

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

    #[cfg(not(kani))]
    listen!(rmi, rmi::REC_ENTER, |arg, ret, rmm| {
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

        #[cfg(not(any(miri, test)))]
        if !crate::gic::validate_state(&run) {
            return Err(Error::RmiErrorRec);
        }

        if rec.host_call_pending() {
            // The below should be called without holding rd's lock
            do_host_call(arg, ret, rmm, &mut rec, &mut run)?;
        }

        #[cfg(not(any(miri, test)))]
        crate::gic::receive_state_from_host(&mut rec, &run)?;
        crate::mmio::emulate_mmio(&mut rec, &run)?;

        crate::rsi::ripas::complete_ripas(&mut rec, &run)?;

        let wfx_flag = run.entry_flags();
        if wfx_flag & (REC_ENTRY_FLAG_TRAP_WFI | REC_ENTRY_FLAG_TRAP_WFE) != 0 {
            warn!("Islet does not support re-configuring the WFI(E) trap");
            warn!("TWI(E) in HCR_EL2 is currently fixed to 'no trap'");
        }

        #[cfg(not(any(miri, test)))]
        activate_stage2_mmu(&rec);

        let mut ret_ns;
        loop {
            ret_ns = true;
            run.set_imm(0);

            rec.set_state(RecState::Running);

            #[cfg(not(any(miri, test)))]
            {
                use crate::rmi::rec::exit::handle_realm_exit;

                let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
                let rd = rd_granule.content::<Rd>()?;

                rec.set_emulatable_abort(NotEmulatableAbort);
                crate::rec::run_prepare(&rd, rec.vcpuid(), &mut rec, 0)?;
                // XXX: we explicitly release Rd's lock here, because RSI calls
                //      would acquire the same lock again (deadlock).
                core::mem::drop(rd_granule);
                match crate::rec::run() {
                    Ok(realm_exit_res) => {
                        (ret_ns, ret[0]) =
                            handle_realm_exit(realm_exit_res, rmm, &mut rec, &mut run)?
                    }
                    Err(_) => ret[0] = rmi::ERROR_REC,
                }
            }

            #[cfg(any(miri, test))]
            {
                use crate::test_utils::mock;
                mock::realm::setup_psci_complete(&mut rec, &mut run);
                mock::realm::setup_ripas_state(&mut rec, &mut run);
            }

            rec.set_state(RecState::Ready);

            if ret_ns {
                break;
            }
        }

        #[cfg(not(any(miri, test)))]
        crate::gic::send_state_to_host(&rec, &mut run)?;
        crate::realm::timer::send_state_to_host(&rec, &mut run)?;

        // NOTICE: do not modify `run` after copy_to_ptr(). it won't have any effect.
        host::copy_to_ptr::<Run>(&run, run_pa).ok_or(Error::RmiErrorInput)
    });

    #[cfg(not(kani))]
    listen!(rmi, rmi::PSCI_COMPLETE, |arg, _ret, _rmm| {
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

#[cfg(test)]
mod test {
    use crate::event::realmexit::RecExitReason;
    use crate::rmi::rec::run::Run;
    use crate::rmi::*;
    use crate::rsi::PSCI_CPU_ON;
    use crate::test_utils::*;

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rec_create
    // Covered RMIs: REC_CREATE, REC_DESTROY, REC_AUX_COUNT
    // Related Spec: D1.2.4 REC creation flow
    #[test]
    fn rmi_rec_create_positive() {
        let rd = realm_create();
        rec_create(rd, IDX_REC1, IDX_REC1_PARAMS, IDX_REC1_AUX);
        rec_destroy(IDX_REC1, IDX_REC1_AUX);
        realm_destroy(rd);

        miri_teardown();
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_psci_complete
    // Covered RMIs: REC_ENTER, PSCI_COMPLETE
    #[test]
    fn rmi_rec_enter_positive() {
        let rd = mock::host::realm_setup();

        let (rec1, run1) = (granule_addr(IDX_REC1), granule_addr(IDX_REC1_RUN));
        let ret = rmi::<REC_ENTER>(&[rec1, run1]);
        assert_eq!(ret[0], SUCCESS);

        unsafe {
            let run = &*(run1 as *const Run);
            let reason: u64 = RecExitReason::PSCI.into();
            assert_eq!(run.exit_reason(), reason as u8);
            assert_eq!(run.gpr(0).unwrap(), PSCI_CPU_ON as u64);
        }

        let rec2 = granule_addr(IDX_REC2);
        const PSCI_E_SUCCESS: usize = 0;
        let ret = rmi::<PSCI_COMPLETE>(&[rec1, rec2, PSCI_E_SUCCESS]);
        assert_eq!(ret[0], SUCCESS);

        mock::host::realm_teardown(rd);

        miri_teardown();
    }
}
