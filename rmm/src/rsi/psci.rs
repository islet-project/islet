use crate::event::RsiHandle;
use crate::granule::GranuleState;
use crate::listen;
use crate::realm::rd::{Rd, State};
use crate::rec::context::{get_reg, set_reg, RegOffset};
use crate::rec::Rec;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmi::rec::mpidr::MPIDR;
use crate::rsi;
use crate::{get_granule, get_granule_if};

struct PsciReturn;
impl PsciReturn {
    const SUCCESS: usize = 0;
    const AFFINITY_INFO_ON: usize = 0;
    const AFFINITY_INFO_OFF: usize = 1;
    const NOT_SUPPORTED: usize = !0;
    const INVALID_PARAMS: usize = !1;
    const DENIED: usize = !2;
    const ALREADY_ON: usize = !3;
    //const ON_PENDING: usize = !4;
    //const INTERNAL_FAILURE: usize = !5;
    //const NOT_PRESENT: usize = !6; // UL(-7)
    //const DISABLED: usize = !7; // UL(-8);
    const INVALID_ADDRESS: usize = !8; //UL(-9);
}

const SMCCC_MAJOR_VERSION: usize = 1;
const SMCCC_MINOR_VERSION: usize = 2;

const PSCI_MAJOR_VERSION: usize = 1;
const PSCI_MINOR_VERSION: usize = 1;

extern crate alloc;

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, rsi::PSCI_VERSION, |_arg, ret, _rmm, rec, _run| {
        if set_reg(rec, 0, psci_version()).is_err() {
            warn!("Unable to set register 0. rec: {:?}", rec);
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, rsi::PSCI_CPU_ON, |_arg, ret, _rmm, rec, run| {
        let rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd.content::<Rd>()?;

        let target_mpidr = get_reg(rec, 1)? as u64;
        let entry_addr = get_reg(rec, 2)?;

        if !rd.addr_in_par(entry_addr) {
            set_reg(rec, 0, PsciReturn::INVALID_ADDRESS)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let target_index = MPIDR::from(target_mpidr).index();
        if target_index >= rd.rec_index() {
            set_reg(rec, 0, PsciReturn::INVALID_PARAMS)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }
        if target_index == rec.vcpuid() {
            set_reg(rec, 0, PsciReturn::ALREADY_ON)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        rec.set_psci_pending(true);
        run.set_exit_reason(rmi::EXIT_PSCI);
        run.set_gpr(0, rsi::PSCI_CPU_ON as u64)?;
        run.set_gpr(1, target_mpidr)?;
        // set 0 for the rest of gprs
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, rsi::PSCI_CPU_OFF, |_arg, ret, _rmm, rec, run| {
        rec.set_runnable(0);
        run.set_exit_reason(rmi::EXIT_PSCI);
        run.set_gpr(0, rsi::PSCI_CPU_OFF as u64)?;
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, rsi::PSCI_SYSTEM_OFF, |_arg, ret, _rmm, rec, run| {
        let mut rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let mut rd = rd.content_mut::<Rd>()?;
        rd.set_state(State::SystemOff);
        run.set_exit_reason(rmi::EXIT_PSCI);
        run.set_gpr(0, rsi::PSCI_SYSTEM_OFF as u64)?;
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, rsi::PSCI_CPU_SUSPEND, |_arg, ret, _rmm, rec, run| {
        let _power_state = get_reg(rec, 1)? as u64;
        let _entry_addr = get_reg(rec, 2)?;
        let _context_id = get_reg(rec, 3)?;

        run.set_exit_reason(rmi::EXIT_PSCI);
        run.set_gpr(0, rsi::PSCI_CPU_SUSPEND as u64)?;
        // set 0 for the rest of gprs
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, rsi::PSCI_SYSTEM_RESET, |_arg, ret, _rmm, rec, run| {
        let mut rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let mut rd = rd.content_mut::<Rd>()?;
        rd.set_state(State::SystemOff);
        run.set_exit_reason(rmi::EXIT_PSCI);
        run.set_gpr(0, rsi::PSCI_SYSTEM_RESET as u64)?;
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, rsi::PSCI_AFFINITY_INFO, |_arg, ret, _rmm, rec, run| {
        #[cfg(feature = "gst_page_table")]
        let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;

        let affinity = get_reg(rec, 1)? as u64;
        let lowest_level = get_reg(rec, 2)?;

        if lowest_level != 0 {
            set_reg(rec, 0, PsciReturn::INVALID_PARAMS)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let target_index = MPIDR::from(affinity).index();
        // check if target_index is in the range
        #[cfg(feature = "gst_page_table")]
        if target_index >= rd_granule.num_children() {
            set_reg(rec, 0, PsciReturn::INVALID_PARAMS)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        if target_index == rec.vcpuid() {
            set_reg(rec, 0, PsciReturn::SUCCESS)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        rec.set_psci_pending(true);
        run.set_exit_reason(rmi::EXIT_PSCI);
        run.set_gpr(0, rsi::PSCI_AFFINITY_INFO as u64)?;
        run.set_gpr(1, affinity)?;
        // set 0 for the rest of gprs
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, rsi::PSCI_FEATURES, |_arg, ret, _rmm, rec, _run| {
        let feature_id = get_reg(rec, 1)?;
        let retval = match feature_id {
            rsi::SMCCC_VERSION //XXX: this should be added for realm-linux booting
            | rsi::PSCI_CPU_SUSPEND
            | rsi::PSCI_CPU_OFF
            | rsi::PSCI_CPU_ON
            | rsi::PSCI_AFFINITY_INFO
            | rsi::PSCI_SYSTEM_OFF
            | rsi::PSCI_SYSTEM_RESET
            | rsi::PSCI_FEATURES
            | rsi::PSCI_VERSION => PsciReturn::SUCCESS,
            _ => PsciReturn::NOT_SUPPORTED,
        };
        if set_reg(rec, 0, retval).is_err() {
            warn!("Unable to set register 0. rec: {:?}", rec);
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, rsi::SMCCC_VERSION, |_arg, ret, _rmm, rec, _run| {
        if set_reg(rec, 0, smccc_version()).is_err() {
            warn!("Unable to set register 0. rec: {:?}", rec);
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });
}

fn psci_version() -> usize {
    (PSCI_MAJOR_VERSION << 16) | PSCI_MINOR_VERSION
}

fn smccc_version() -> usize {
    (SMCCC_MAJOR_VERSION << 16) | SMCCC_MINOR_VERSION
}

pub fn complete_psci(
    caller: &mut Rec<'_>,
    target: &mut Rec<'_>,
    status: usize,
) -> Result<(), Error> {
    let target_vcpuid = target.vcpuid();

    let target_mpidr = get_reg(caller, 1)? as u64;
    // TODO: check the below again
    if MPIDR::from(target_mpidr).index() != target_vcpuid {
        return Err(Error::RmiErrorInput);
    }

    let command = get_reg(caller, 0)?;
    match command {
        rsi::PSCI_CPU_ON => {
            if status != PsciReturn::SUCCESS && status != PsciReturn::DENIED {
                return Err(Error::RmiErrorInput);
            }
        }
        rsi::PSCI_AFFINITY_INFO => {
            if status != PsciReturn::SUCCESS {
                return Err(Error::RmiErrorInput);
            }
        }
        _ => {}
    }

    let psci_ret = match command {
        rsi::PSCI_CPU_ON if target.runnable() => PsciReturn::ALREADY_ON,
        rsi::PSCI_CPU_ON if status == PsciReturn::DENIED => PsciReturn::DENIED,
        rsi::PSCI_CPU_ON => {
            let entry_point = get_reg(caller, 2)?;
            let context_id = get_reg(caller, 3)?;
            target.reset_ctx();
            set_reg(target, 0, context_id)?;
            set_reg(target, RegOffset::PC, entry_point)?;
            target.set_runnable(1);
            PsciReturn::SUCCESS
        }
        rsi::PSCI_AFFINITY_INFO if target.runnable() => PsciReturn::AFFINITY_INFO_ON,
        rsi::PSCI_AFFINITY_INFO => PsciReturn::AFFINITY_INFO_OFF,
        _ => PsciReturn::NOT_SUPPORTED,
    };

    if command == rsi::PSCI_CPU_ON
        && status == PsciReturn::DENIED
        && psci_ret == PsciReturn::ALREADY_ON
    {
        return Err(Error::RmiErrorInput);
    }

    set_reg(caller, 0, psci_ret)?;
    set_reg(caller, 1, 0)?;
    set_reg(caller, 2, 0)?;
    set_reg(caller, 3, 0)?;
    caller.set_psci_pending(false);
    Ok(())
}
