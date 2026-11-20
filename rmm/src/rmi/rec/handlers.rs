use super::mpidr::MPIDR;
use super::params::Params;
use super::run::{EntryFlag, Run};
use super::vtcr::{activate_stage2_mmu, prepare_vtcr};
use crate::event::RmiHandle;
#[cfg(feature = "gst_page_table")]
use crate::granule::{is_not_in_realm, set_granule, set_granule_with_parent, GranuleState};
#[cfg(not(feature = "gst_page_table"))]
use crate::granule::{is_not_in_realm, set_granule, GranuleState};
use crate::host;
use crate::listen;
use crate::measurement::HashContext;
#[cfg(not(kani))]
use crate::mm::translation::PageTable;
use crate::realm::rd::{Rd, State};
use crate::rec::context::{set_reg, RegOffset};
use crate::rec::simd::aux_granules_needed;
use crate::rec::State as RecState;
use crate::rec::{max_recs_order, Rec, RmmRecEmulatableAbort::NotEmulatableAbort};
use crate::rmi;
use crate::rmi::error::Error;
use crate::rsi::do_host_call;
use crate::rsi::psci::complete_psci;
use crate::{get_granule, get_granule_if};

use aarch64_cpu::registers::*;
use armv9a::bits_in_reg;

extern crate alloc;

fn prepare_args(rd: &Rd, mpidr: u64) -> Result<(usize, u64, u64), Error> {
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
    Ok((vcpuid, vttbr, vmpidr))
}

#[cfg(not(kani))]
fn transition_aux_granules(
    aux_addrs: &[u64],
    num_aux: usize,
    transitioned: &mut usize,
) -> Result<(), Error> {
    for &aux in &aux_addrs[..num_aux] {
        let mut aux_granule = get_granule_if!(aux as usize, GranuleState::Delegated)?;
        set_granule(&mut aux_granule, GranuleState::RecAux)?;
        *transitioned += 1;
    }
    Ok(())
}

#[cfg(not(kani))]
fn revert_aux_granules(page_table: &PageTable, aux_addrs: &[u64], count: usize) {
    // Map each AUX granule at its identity address before transitioning back
    // to Delegated so Granule::zeroize() can wipe it safely.
    for &aux in &aux_addrs[..count] {
        let aux = aux as usize;
        if let Ok(mut aux_granule) = get_granule_if!(aux, GranuleState::RecAux) {
            page_table.map(aux, true);
            let _ = set_granule(&mut aux_granule, GranuleState::Delegated);
            page_table.unmap(aux);
        }
    }
}

#[cfg(not(kani))]
#[allow(clippy::too_many_arguments)]
fn init_rec_state(
    rec: &mut Rec<'_>,
    rd: &mut Rd,
    params: &Params,
    page_table: &PageTable,
    rd_addr: usize,
    vcpuid: usize,
    vttbr: u64,
    vmpidr: u64,
    vtcr: u64,
    num_aux: usize,
) -> Result<(), Error> {
    // Setup must happen before Rec::init() so the contiguous AUX mapping and
    // bump allocator are initialized before init_simd() allocates from them.
    rec.setup_aux_memory(page_table, &params.aux, num_aux)?;
    rec.init(rd_addr, vcpuid, params.flags, params.aux, vttbr, vmpidr)?;

    for (idx, gpr) in params.gprs.iter().enumerate() {
        if set_reg(rec, idx, *gpr as usize).is_err() {
            return Err(Error::RmiErrorInput);
        }
    }
    if set_reg(rec, RegOffset::PC, params.pc as usize).is_err() {
        return Err(Error::RmiErrorInput);
    }
    rec.set_vtcr(vtcr);

    HashContext::new(rd)?.measure_rec_params(params)?;
    Ok(())
}

#[cfg(not(kani))]
#[allow(clippy::too_many_arguments)]
fn create_rec_granule(
    #[cfg(feature = "gst_page_table")] rd_parent: &crate::granule::page_table::entry::Inner,
    rd: &mut Rd,
    rd_addr: usize,
    rec_addr: usize,
    params: &Params,
    page_table: &PageTable,
    vcpuid: usize,
    vttbr: u64,
    vmpidr: u64,
    vtcr: u64,
    num_aux: usize,
) -> Result<(), Error> {
    let mut rec_granule = get_granule_if!(rec_addr, GranuleState::Delegated)?;
    page_table.map(rec_addr, true);

    let mut rec_transitioned = false;
    #[cfg(feature = "gst_page_table")]
    let res = set_granule_with_parent(rd_parent.clone(), &mut rec_granule, GranuleState::Rec);
    #[cfg(not(feature = "gst_page_table"))]
    let res = set_granule(&mut rec_granule, GranuleState::Rec);

    let res = res.and_then(|()| {
        rec_transitioned = true;
        let mut rec = rec_granule.new_uninit_with::<Rec<'_>>(Rec::new())?;
        let init_res = init_rec_state(
            &mut rec, rd, params, page_table, rd_addr, vcpuid, vttbr, vmpidr, vtcr, num_aux,
        );
        if init_res.is_err() {
            rec.cleanup_aux_memory(page_table);
        }
        init_res
    });

    if let Err(err) = res {
        if rec_transitioned {
            // Move rec_granule back to Delegated state (also zeroizes the Rec struct).
            let _ = set_granule(&mut rec_granule, GranuleState::Delegated);
        }
        page_table.unmap(rec_addr);
        return Err(err);
    }

    Ok(())
}

pub fn set_event_handler(rmi: &mut RmiHandle) {
    #[cfg(not(kani))]
    listen!(rmi, rmi::REC_CREATE, |arg, ret, rmm| {
        let rd_addr = arg[0];
        let rec_addr = arg[1];
        let params_addr = arg[2];

        if rec_addr == rd_addr || rec_addr == params_addr || rd_addr == params_addr {
            return Err(Error::RmiErrorInput);
        }

        if !is_not_in_realm(params_addr) {
            return Err(Error::RmiErrorInput);
        }

        rmm.page_table.map(params_addr, false);
        let params = host::copy_from::<Params>(params_addr);
        rmm.page_table.unmap(params_addr);
        let params = params.ok_or(Error::RmiErrorInput)?;
        params.verify_compliance(rec_addr, rd_addr, params_addr)?;

        let rec_index = MPIDR::from(params.mpidr).index();
        let mut rd_granule = get_granule_if!(rd_addr, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }
        let max_recs = 1 << max_recs_order();
        if rd.num_recs() >= max_recs || rec_index >= max_recs {
            return Err(Error::RmiErrorRealm(0));
        }

        if rec_index != rd.rec_index() {
            return Err(Error::RmiErrorInput);
        }

        // The host must supply the exact number of AUX granules required for
        // the REC (currently sized to hold SimdRegister; future features may
        // require additional AUX granules).
        let expected_aux_count = aux_granules_needed(rd.simd_config());
        let num_aux = params.num_aux as usize;
        if num_aux != expected_aux_count {
            error!(
                "expected_aux_count {} != params.num_aux {}",
                expected_aux_count, params.num_aux
            );
            return Err(Error::RmiErrorInput);
        }

        // Compute read-only RD/VTCR values before modifying page tables or granule states.
        let (vcpuid, vttbr, vmpidr) = prepare_args(&rd, params.mpidr)?;
        let vtcr = prepare_vtcr(&rd)?;

        // Transition auxiliary granules from Delegated to RecAux before locking
        // rec_granule so we never hold rec_granule and an aux_granule lock at
        // the same time (preventing AB-BA deadlocks across concurrent calls).
        let mut transitioned_aux = 0usize;
        if let Err(err) = transition_aux_granules(&params.aux, num_aux, &mut transitioned_aux) {
            revert_aux_granules(&rmm.page_table, &params.aux, transitioned_aux);
            return Err(err);
        }

        if let Err(err) = create_rec_granule(
            #[cfg(feature = "gst_page_table")]
            &rd_granule,
            &mut rd,
            rd_addr,
            rec_addr,
            &params,
            &rmm.page_table,
            vcpuid,
            vttbr,
            vmpidr,
            vtcr,
            num_aux,
        ) {
            revert_aux_granules(&rmm.page_table, &params.aux, transitioned_aux);
            return Err(err);
        }

        ret[1] = vcpuid;
        rd.vcpu_index += 1;
        rd.inc_recs();

        #[cfg(not(feature = "gst_page_table"))]
        rd_granule.inc_count();

        Ok(())
    });

    #[cfg(any(not(kani), feature = "mc_rmi_rec_destroy"))]
    listen!(rmi, rmi::REC_DESTROY, |arg, _ret, rmm| {
        let mut rec_granule = get_granule_if!(arg[0], GranuleState::Rec)?;

        let mut rec = rec_granule.content_mut::<Rec<'_>>()?;
        if rec.get_state() == RecState::Running {
            return Err(Error::RmiErrorRec);
        }

        let rd = rec.owner()?;

        // Unmap the contiguous multi-granule mapping of the AUX granules
        // and transition them back to Delegated. Validate all AUX granules
        // upfront before unmapping or zeroizing so an error cannot leave
        // the REC/AUX state partially torn down. Map each AUX granule at its
        // identity address while transitioning to Delegated so Granule::zeroize()
        // can wipe it safely.
        #[cfg(not(kani))]
        {
            let num_granules = rec.num_aux_granules();
            for i in 0..num_granules {
                let rec_aux = rec.aux(i) as usize;
                let _ = get_granule_if!(rec_aux, GranuleState::RecAux)?;
            }

            rec.cleanup_aux_memory(&rmm.page_table);
            for i in 0..num_granules {
                let rec_aux = rec.aux(i) as usize;
                let mut rec_aux_granule = get_granule_if!(rec_aux, GranuleState::RecAux)?;
                rmm.page_table.map(rec_aux, true);
                let res = set_granule(&mut rec_aux_granule, GranuleState::Delegated);
                rmm.page_table.unmap(rec_aux);
                res?;
            }
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

        #[cfg(kani)]
        {
            // XXX: the below can be guaranteed by Rec's invariants instead
            kani::assume(crate::granule::validate_addr(rd));
            let rd_granule = get_granule!(rd)?;
            kani::assume(rd_granule.state() == GranuleState::RD);
        }
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        #[cfg(not(feature = "gst_page_table"))]
        rd_granule.dec_count();
        let mut rd = rd_granule.content::<Rd>()?;
        rd.dec_recs();

        set_granule(&mut rec_granule, GranuleState::Delegated).inspect_err(|_| {
            #[cfg(not(kani))]
            // `page_table` is currently not reachable in model checking harnesses
            rmm.page_table.unmap(arg[0]);
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

        if !is_not_in_realm(run_pa) {
            return Err(Error::RmiErrorInput);
        }

        // read Run
        rmm.page_table.map(run_pa, false);
        let run_copy = host::copy_from::<Run>(run_pa);
        rmm.page_table.unmap(run_pa);
        let mut run = run_copy.ok_or(Error::RmiErrorInput)?;
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

        #[cfg(not(any(miri, test, fuzzing)))]
        if !crate::rec::gic::validate_state(&run) {
            return Err(Error::RmiErrorRec);
        }

        if rec.host_call_pending() {
            // The below should be called without holding rd's lock
            do_host_call(arg, ret, rmm, &mut rec, &mut run)?;
        }

        #[cfg(not(any(miri, test, fuzzing)))]
        crate::rec::gic::receive_state_from_host(&mut rec, &run)?;
        crate::rec::mmio::emulate_mmio(&mut rec, &run)?;
        crate::rec::sea::host_sea_inject(&mut rec, &run)?;

        crate::rsi::ripas::complete_ripas(&mut rec, &run)?;

        let wfx_flag = run.entry_flags();
        #[cfg(not(any(miri, test, fuzzing)))]
        {
            let hcr_el2 = HCR_EL2.get();
            let mut wfx = 0;
            let wfx_mask = !(3 << 13);
            if wfx_flag.get_masked(EntryFlag::TRAP_WFI) != 0 {
                wfx |= 1 << 13;
            }
            if wfx_flag.get_masked(EntryFlag::TRAP_WFE) != 0 {
                wfx |= 1 << 14;
            }
            HCR_EL2.set((hcr_el2 & wfx_mask) | wfx);
        }

        #[cfg(not(any(miri, test, fuzzing)))]
        activate_stage2_mmu(&rec);

        crate::rec::save_host_state(&rec);
        let mut ret_ns;
        loop {
            ret_ns = true;
            run.set_imm(0);

            rec.set_state(RecState::Running);

            #[cfg(not(any(miri, test, fuzzing)))]
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

            #[cfg(fuzzing)]
            {
                use crate::test_utils::mock;

                // In fuzzing contexts, unused register x3 denotes either RSI command or
                // pseudo RSI command, REC_ENTER_EXIT_CMD for simulating realm exit conditions.
                // Unused registers x4 and above serve as arguments for the RSI command.
                if arg.len() >= 3 {
                    let cmd = arg[2];
                    let args = &arg[3..];

                    rec.set_emulatable_abort(NotEmulatableAbort);

                    (_, ret[0]) = mock::realm::emulate_realm(rmm, &mut rec, &mut run, cmd, args)?;
                }
            }

            rec.set_state(RecState::Ready);

            if ret_ns {
                break;
            }
        }

        #[cfg(not(any(miri, test, fuzzing)))]
        crate::rec::gic::send_state_to_host(&rec, &mut run)?;
        crate::rec::timer::send_state_to_host(&rec, &mut run)?;
        crate::rec::pmu::send_state_to_host(&rec, &mut run)?;
        crate::rec::restore_host_state(&rec);

        // NOTICE: do not modify `run` after copy_to_ptr(). it won't have any effect.
        rmm.page_table.map(run_pa, false);
        let ret = host::copy_to_ptr::<Run>(&run, run_pa).ok_or(Error::RmiErrorInput);
        rmm.page_table.unmap(run_pa);
        ret
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
        rec_destroy(rd, IDX_REC1, IDX_REC1_AUX);
        realm_destroy(rd);

        miri_teardown();
    }

    #[test]
    fn rmi_rec_create_rollback_on_error() {
        use crate::granule::{GranuleState, GRANULE_SIZE};
        use crate::realm::rd::Rd;
        use crate::rmi::rec::params::Params as RecParams;
        use crate::{get_granule, get_granule_if};

        let rd = realm_create();
        let (rec, params_ptr) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_PARAMS));

        // The mock granule region is shared by all the unit tests and, in test/miri
        // builds, granule zeroing on the Delegated/Undelegated transitions only wipes
        // the first 8 bytes (see granule::array::entry::zeroize()). So the REC granule
        // may still hold the stale `Rec` image written by a previously executed test,
        // including its padding bytes which are uninitialized from MIRI's point of
        // view. `Params` models its layout padding as real `[u8; N]` fields, so reading
        // those bytes back as a `Params` (the host::copy_from() below) would be UB.
        // Initialize the whole granule that is passed as a bogus params pointer.
        unsafe {
            core::ptr::write_bytes(rec as *mut u8, 0, GRANULE_SIZE);
        }

        // Aliasing params_ptr with rd/rec, or passing an out-of-NS-DRAM / unaligned
        // params_ptr must fail before mapping without unmapping rd.
        assert_eq!(rmi::<REC_CREATE>(&[rd, rec, rd])[0], ERROR_INPUT);
        assert_eq!(rmi::<REC_CREATE>(&[rd, rec, rec])[0], ERROR_INPUT);
        assert_eq!(rmi::<REC_CREATE>(&[rd, rec, 0x1000])[0], ERROR_INPUT);
        assert_eq!(
            rmi::<REC_CREATE>(&[rd, rec, params_ptr + 1])[0],
            ERROR_INPUT
        );

        assert_eq!(rmi::<GRANULE_DELEGATE>(&[rec])[0], SUCCESS);

        let ret = rmi::<REC_AUX_COUNT>(&[rd]);
        assert_eq!(ret[0], SUCCESS);
        let aux_count = ret[1];

        unsafe {
            let params = &mut *(params_ptr as *mut RecParams);
            *params = RecParams::default();
            params.pc = 0x1000;
            params.flags = 1;
            params.mpidr = 0;
            params.num_aux = aux_count as u64;
            for idx in 0..aux_count {
                let aux_addr = alloc_granule(idx + IDX_REC1_AUX);
                assert_eq!(rmi::<GRANULE_DELEGATE>(&[aux_addr])[0], SUCCESS);
                params.aux[idx] = aux_addr as u64;
            }
        }

        // Force failure at the very end of init_res (HashContext::new) after
        // rec_granule and all AUX granules have been transitioned and written.
        let saved_algo = unsafe {
            let rd_obj = &mut *(rd as *mut Rd);
            let old = rd_obj.hash_algo();
            rd_obj.set_hash_algo(u8::MAX);
            old
        };

        let ret = rmi::<REC_CREATE>(&[rd, rec, params_ptr]);
        assert_ne!(ret[0], SUCCESS);

        // Verify rec and all aux granules were rolled back to Delegated and wiped,
        // and RD counters were not incremented.
        {
            let rec_granule = get_granule_if!(rec, GranuleState::Delegated).unwrap();
            assert_eq!(rec_granule.state(), GranuleState::Delegated);
            unsafe {
                assert_eq!(*(rec as *const u64), 0);
            }
        }
        for idx in 0..aux_count {
            let aux_addr = granule_addr(idx + IDX_REC1_AUX);
            let aux_granule = get_granule_if!(aux_addr, GranuleState::Delegated).unwrap();
            assert_eq!(aux_granule.state(), GranuleState::Delegated);
            unsafe {
                let slice = core::slice::from_raw_parts(aux_addr as *const u8, GRANULE_SIZE);
                assert_eq!(&slice[..8], &[0u8; 8]);
            }
        }
        unsafe {
            let rd_obj = &mut *(rd as *mut Rd);
            assert_eq!(rd_obj.num_recs(), 0);
            assert_eq!(rd_obj.vcpu_index, 0);
            rd_obj.set_hash_algo(saved_algo);
        }

        // Now REC_CREATE with the same granules must succeed cleanly
        let ret = rmi::<REC_CREATE>(&[rd, rec, params_ptr]);
        assert_eq!(ret[0], SUCCESS);

        rec_destroy(rd, IDX_REC1, IDX_REC1_AUX);
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
