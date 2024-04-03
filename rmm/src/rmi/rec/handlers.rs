use super::mpidr::MPIDR;
use super::params::Params;
use super::run::{Run, REC_ENTRY_FLAG_TRAP_WFE, REC_ENTRY_FLAG_TRAP_WFI};
use super::vtcr::{activate_stage2_mmu, prepare_vtcr};
use super::Rec;
use crate::event::Mainloop;
#[cfg(feature = "gst_page_table")]
use crate::granule::{set_granule, set_granule_with_parent, GranuleState};
#[cfg(not(feature = "gst_page_table"))]
use crate::granule::{set_granule, GranuleState};
use crate::host;
use crate::listen;
use crate::measurement::HashContext;
use crate::realm::context::set_reg;
use crate::realm::vcpu::create_vcpu;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmi::realm::{rd::State, Rd};
use crate::rmi::rec::exit::handle_realm_exit;
use crate::rmi::rec::RecState;
use crate::rsi::do_host_call;
use crate::{get_granule, get_granule_if};

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REC_CREATE, |arg, ret, rmm| {
        let rec = arg[0];
        let rd = arg[1];
        let params_ptr = arg[2];
        let owner = rd;

        if rec == rd {
            return Err(Error::RmiErrorInput);
        }

        let params = host::copy_from::<Params>(params_ptr).ok_or(Error::RmiErrorInput)?;
        params.verify_compliance(rec, rd, params_ptr)?;

        let rec_index = MPIDR::from(params.mpidr).index();
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let rd = rd_granule.content_mut::<Rd>();
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }

        if rec_index != rd.rec_index() {
            return Err(Error::RmiErrorInput);
        }
        // set Rec_state and grab the lock for Rec granule
        let mut rec_granule = get_granule_if!(rec, GranuleState::Delegated)?;
        rmm.page_table.map(rec, true);
        let rec = rec_granule.content_mut::<Rec<'_>>();

        match create_vcpu(rd.id()) {
            Ok(vcpuid) => {
                ret[1] = vcpuid;
                rec.init(owner, vcpuid, params.flags)?;
            }
            Err(_) => return Err(Error::RmiErrorInput),
        }

        for (idx, gpr) in params.gprs.iter().enumerate() {
            if set_reg(rd.id(), rec.vcpuid(), idx, *gpr as usize).is_err() {
                return Err(Error::RmiErrorInput);
            }
        }
        if set_reg(rd.id(), rec.vcpuid(), 31, params.pc as usize).is_err() {
            return Err(Error::RmiErrorInput);
        }
        rec.set_vtcr(prepare_vtcr(rd)?);

        rd.inc_rec_index();
        HashContext::new(rd)?.measure_rec_params(&params)?;

        #[cfg(feature = "gst_page_table")]
        return set_granule_with_parent(rd_granule.clone(), &mut rec_granule, GranuleState::Rec);
        #[cfg(not(feature = "gst_page_table"))]
        return set_granule(&mut rec_granule, GranuleState::Rec);
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
        let run_pa = arg[1];

        // grab the lock for Rec
        let mut rec_granule = get_granule_if!(arg[0], GranuleState::Rec)?;
        let rec = rec_granule.content_mut::<Rec<'_>>();
        let realm_id = rec.realmid()?;

        if !rec.runnable() {
            return Err(Error::RmiErrorRec);
        }

        if let RecState::Running = rec.get_state() {
            error!("Rec is already running: {:?}", rec);
            return Err(Error::RmiErrorRec);
        }

        match get_granule_if!(rec.owner()?, GranuleState::RD)?
            .content::<Rd>()
            .state() // Rd dropped
        {
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

        // read Run
        let mut run = host::copy_from::<Run>(run_pa).ok_or(Error::RmiErrorInput)?;
        run.verify_compliance()?;
        trace!("{:?}", run);

        if rec.host_call_pending() {
            do_host_call(arg, ret, rmm, rec, &mut run)?;
        }

        crate::gic::receive_state_from_host(realm_id, rec.vcpuid(), &run)?;
        crate::mmio::emulate_mmio(realm_id, rec.vcpuid(), &run)?;

        let ripas = rec.ripas_addr() as usize;
        if ripas > 0 {
            set_reg(realm_id, rec.vcpuid(), 0, 0)?;
            set_reg(realm_id, rec.vcpuid(), 1, ripas)?;
            rec.set_ripas(0, 0, 0, 0);
        }

        let wfx_flag = run.entry_flags();
        if wfx_flag & (REC_ENTRY_FLAG_TRAP_WFI | REC_ENTRY_FLAG_TRAP_WFE) != 0 {
            warn!("Islet does not support re-configuring the WFI(E) trap");
            warn!("TWI(E) in HCR_EL2 is currently fixed to 'no trap'");
        }

        activate_stage2_mmu(rec);

        let mut ret_ns;
        loop {
            ret_ns = true;
            run.set_imm(0);

            rec.set_state(RecState::Running);
            match crate::rmi::rec::run(realm_id, rec.vcpuid(), 0) {
                Ok(realm_exit_res) => {
                    (ret_ns, ret[0]) = handle_realm_exit(realm_exit_res, rmm, rec, &mut run)?
                }
                Err(_) => ret[0] = rmi::ERROR_REC,
            }
            rec.set_state(RecState::Ready);

            if ret_ns {
                break;
            }
        }
        crate::gic::send_state_to_host(realm_id, rec.vcpuid(), &mut run)?;
        crate::realm::timer::send_state_to_host(realm_id, rec.vcpuid(), &mut run)?;

        // NOTICE: do not modify `run` after copy_to_host_or_ret!(). it won't have any effect.
        host::copy_to::<Run>(&run, run_pa).ok_or(Error::RmiErrorInput)
    });
}
