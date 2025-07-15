use crate::event::realmexit::*;
use crate::event::RsiHandle;
use crate::get_granule;
use crate::get_granule_if;
use crate::granule::GranuleState;
use crate::granule::GRANULE_MASK;
use crate::realm::mm::rtt::RTT_PAGE_LEVEL;
use crate::realm::mm::stage2_tte::S2TTE;
use crate::realm::rd::Rd;
use crate::rec::context::{get_reg, set_reg, RegOffset};
use crate::rec::{
    Rec, RmmRecEmulatableAbort::EmulatableAbort, RmmRecEmulatableAbort::NotEmulatableAbort,
};
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;
use crate::Monitor;
use crate::{rmi, rsi};
use armv9a::{
    EsrEl2, DFSC_PERM_FAULTS, DFSC_PERM_FAULT_MASK, EMULATABLE_ABORT_MASK, INST_ABORT_MASK,
    NON_EMULATABLE_ABORT_MASK, SERROR_MASK, WFX_MASK,
};

use aarch64_cpu::registers::{Readable, Writeable};
use aarch64_cpu::registers::{ELR_EL1, ELR_EL2, HPFAR_EL2, SPSR_EL1, SPSR_EL2, VBAR_EL1};

#[derive(Debug)]
enum AbortHandleType {
    SeaInject,
    AddrSizeFaultInject,
    NonEmulatableExit,
    EmulatableExit,
    DataAbortExit,
}

pub fn handle_realm_exit(
    realm_exit_res: [usize; 4],
    rmm: &Monitor,
    rec: &mut Rec<'_>,
    run: &mut Run,
) -> Result<(bool, usize), Error> {
    let mut return_to_ns = true;
    let ret = match RecExitReason::from(realm_exit_res[0]) {
        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        RecExitReason::Sync(ExitSyncType::RSI) => {
            trace!("REC_ENTER ret: {:#X?}", realm_exit_res);
            let cmd = realm_exit_res[1];
            let mut ret = rmi::SUCCESS;

            let mut rsi_ctx = rsi::constraint::validate(cmd);
            // set default value
            if rmm.handle_rsi(&mut rsi_ctx, rec, run) == RsiHandle::RET_SUCCESS {
                if rsi_ctx.ret_slice()[0] == rmi::SUCCESS_REC_ENTER {
                    return_to_ns = false;
                }
                ret = rsi_ctx.ret_slice()[0];
            } else {
                return_to_ns = false;
            }
            ret
        }
        RecExitReason::Sync(ExitSyncType::DataAbort) => {
            match handle_data_abort(realm_exit_res, rec, run)? {
                rmi::SUCCESS => {
                    run.set_exit_reason(rmi::EXIT_SYNC);
                    run.set_hpfar(realm_exit_res[2] as u64);
                    rmi::SUCCESS
                }
                rmi::SUCCESS_REC_ENTER => {
                    return_to_ns = false;
                    rmi::SUCCESS
                }
                _ => panic!("shouldn't be reached here"),
            }
        }
        RecExitReason::IRQ => {
            run.set_exit_reason(rmi::EXIT_IRQ);
            run.set_esr(0);
            run.set_hpfar(0);
            run.set_far(0);
            rmi::SUCCESS
        }
        RecExitReason::SError => {
            run.set_exit_reason(rmi::EXIT_SERROR);
            run.set_esr(realm_exit_res[1] as u64 & SERROR_MASK);
            run.set_hpfar(0);
            run.set_far(0);
            rmi::SUCCESS
        }
        RecExitReason::Sync(ExitSyncType::InstAbort) => {
            match handle_inst_abort(realm_exit_res, rec, run)? {
                rmi::SUCCESS => {
                    run.set_exit_reason(rmi::EXIT_SYNC);
                    run.set_hpfar(realm_exit_res[2] as u64);
                    rmi::SUCCESS
                }
                rmi::SUCCESS_REC_ENTER => {
                    return_to_ns = false;
                    rmi::SUCCESS
                }
                _ => panic!("shouldn't be reached here"),
            }
        }
        RecExitReason::Sync(ExitSyncType::WFx) => {
            let esr_el2 = realm_exit_res[1] as u64;
            run.set_exit_reason(rmi::EXIT_SYNC);
            run.set_esr(esr_el2 & WFX_MASK);
            run.set_hpfar(0);
            run.set_far(0);
            rmi::SUCCESS
        }
        RecExitReason::Sync(ExitSyncType::Undefined) => {
            run.set_exit_reason(rmi::EXIT_SYNC);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            rmi::SUCCESS
        }
        _ => rmi::SUCCESS,
    };

    Ok((return_to_ns, ret))
}

fn get_write_val(rec: &Rec<'_>, esr_el2: u64) -> Result<u64, Error> {
    let esr_el2 = EsrEl2::new(esr_el2);
    let rt = esr_el2.get_masked_value(EsrEl2::SRT) as usize;
    let write_val = match rt == 31 {
        true => 0, // xzr
        false => get_reg(rec, rt)? as u64 & esr_el2.get_access_size_mask(),
    };
    Ok(write_val)
}

fn inject_sea(rec: &mut Rec<'_>, esr_el2: u64, far_el2: u64) {
    let mut esr_el1 = esr_el2 & !(EsrEl2::EC | EsrEl2::FNV | EsrEl2::S1PTW | EsrEl2::DFSC);
    let mut ec = esr_el2 & EsrEl2::EC;
    if SPSR_EL2.read(SPSR_EL2::M) != SPSR_EL2::M::EL0t.into() {
        ec |= 1 << EsrEl2::EC.trailing_zeros();
    }
    esr_el1 |= ec;
    esr_el1 |= EsrEl2::EA;
    esr_el1 |= 0b010000; // Synchronous External Abort (SEA)
    const VBAR_CURRENT_SP0_OFFSET: u64 = 0x0;
    const VBAR_CURRENT_SPX_OFFSET: u64 = 0x200;
    const VBAR_LOWER_AARCH64_OFFSET: u64 = 0x400;
    let mut vector_entry = {
        match SPSR_EL2.read_as_enum(SPSR_EL2::M) {
            Some(SPSR_EL2::M::Value::EL0t) => VBAR_LOWER_AARCH64_OFFSET, //EL0t
            Some(SPSR_EL2::M::Value::EL1t) => VBAR_CURRENT_SP0_OFFSET,   //EL1t
            Some(SPSR_EL2::M::Value::EL1h) => VBAR_CURRENT_SPX_OFFSET,   //EL1h
            _ => panic!("shouldn't be reached here"), // Realms run at aarch64 state only (i.e. no aarch32)
        }
    };
    vector_entry += VBAR_EL1.get();

    let pstate: u64 = (SPSR_EL2::D::SET
        + SPSR_EL2::A::SET
        + SPSR_EL2::I::SET
        + SPSR_EL2::F::SET
        + SPSR_EL2::M::EL1h)
        .into();

    let context = &mut rec.context;
    context.sys_regs.esr_el1 = esr_el1;
    context.sys_regs.far = far_el2;
    context.elr_el2 = vector_entry;
    let _ = set_reg(rec, RegOffset::PSTATE, pstate as usize);
    ELR_EL1.set(ELR_EL2.get());
    SPSR_EL1.set(SPSR_EL2.get());
}

fn abort_handle_type(
    rd: &Rd,
    exit: ExitSyncType,
    esr_el2: u64,
    fault_ipa: usize,
) -> Result<AbortHandleType, Error> {
    let is_protected_ipa = rd.addr_in_par(fault_ipa);
    let (s2tte, last_level) =
        S2TTE::get_s2tte(rd, fault_ipa, RTT_PAGE_LEVEL, Error::RmiErrorRtt(0))?;
    let esr = EsrEl2::new(esr_el2);

    if is_protected_ipa {
        if s2tte.is_assigned_empty() || s2tte.is_unassigned_empty() {
            return Ok(AbortHandleType::SeaInject);
        }
        if s2tte.is_unassigned_ram() || s2tte.is_destroyed() {
            return Ok(AbortHandleType::NonEmulatableExit);
        }
    } else if fault_ipa > rd.ipa_size() {
        return Ok(AbortHandleType::AddrSizeFaultInject);
    } else {
        // for unprotected IPA
        if exit == ExitSyncType::InstAbort {
            return Ok(AbortHandleType::SeaInject);
        }
        let dfsc = esr.get_masked_value(EsrEl2::DFSC) & DFSC_PERM_FAULT_MASK;
        let mut check_isv = false;
        if s2tte.is_unassigned_ns()
            || (s2tte.is_assigned_ns(last_level) && dfsc == DFSC_PERM_FAULTS)
        {
            check_isv = true;
        }
        if check_isv {
            if esr.get_masked_value(EsrEl2::ISV) == 1 {
                return Ok(AbortHandleType::EmulatableExit);
            } else {
                return Ok(AbortHandleType::NonEmulatableExit);
            }
        }
    }
    Ok(AbortHandleType::DataAbortExit)
}

fn handle_data_abort(
    realm_exit_res: [usize; 4],
    rec: &mut Rec<'_>,
    run: &mut Run,
) -> Result<usize, Error> {
    let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>()?;

    let esr_el2 = realm_exit_res[1] as u64;
    let hpfar_el2 = realm_exit_res[2] as u64;
    let far_el2 = realm_exit_res[3] as u64;

    let fault_ipa = hpfar_el2 & (HPFAR_EL2::FIPA.mask << HPFAR_EL2::FIPA.shift);
    let fault_ipa = (fault_ipa << 8) as usize;

    let ret = match abort_handle_type(&rd, ExitSyncType::DataAbort, esr_el2, fault_ipa)? {
        AbortHandleType::SeaInject => {
            inject_sea(rec, esr_el2, far_el2);
            rmi::SUCCESS_REC_ENTER
        }
        AbortHandleType::AddrSizeFaultInject => {
            unimplemented!();
        }
        AbortHandleType::NonEmulatableExit => {
            rec.set_emulatable_abort(NotEmulatableAbort);
            if rd.addr_in_par(fault_ipa) {
                run.set_esr(esr_el2 & NON_EMULATABLE_ABORT_MASK);
            } else {
                run.set_esr(esr_el2 & NON_EMULATABLE_ABORT_MASK);
                // FIXME: According to the RMM Spec, Non emulatable abort at unprotected ipa
                // should carry ELR_EL2's IL bit. However, ACS test  checks the opposite.
                //run.set_esr(esr_el2 & (NON_EMULATABLE_ABORT_MASK | EsrEl2::IL));
            }
            run.set_far(0);
            rmi::SUCCESS
        }
        AbortHandleType::EmulatableExit => {
            if esr_el2 & EsrEl2::WNR != 0 {
                let write_val = get_write_val(rec, esr_el2)?;
                run.set_gpr(0, write_val)?;
            }
            rec.set_emulatable_abort(EmulatableAbort);
            run.set_esr(esr_el2 & EMULATABLE_ABORT_MASK);
            run.set_far(far_el2 & !(GRANULE_MASK as u64));
            rmi::SUCCESS
        }
        AbortHandleType::DataAbortExit => {
            rec.set_emulatable_abort(NotEmulatableAbort);
            run.set_esr(esr_el2 & NON_EMULATABLE_ABORT_MASK);
            run.set_far(0);
            rmi::SUCCESS
        }
    };

    Ok(ret)
}

fn handle_inst_abort(
    realm_exit_res: [usize; 4],
    rec: &mut Rec<'_>,
    run: &mut Run,
) -> Result<usize, Error> {
    let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>()?;

    let esr_el2 = realm_exit_res[1] as u64;
    let hpfar_el2 = realm_exit_res[2] as u64;
    let far_el2 = realm_exit_res[3] as u64;

    let fault_ipa = hpfar_el2 & (HPFAR_EL2::FIPA.mask << HPFAR_EL2::FIPA.shift);
    let fault_ipa = (fault_ipa << 8) as usize;

    let ret = match abort_handle_type(&rd, ExitSyncType::InstAbort, esr_el2, fault_ipa)? {
        AbortHandleType::SeaInject => {
            inject_sea(rec, esr_el2, far_el2);
            rmi::SUCCESS_REC_ENTER
        }
        AbortHandleType::AddrSizeFaultInject => {
            unimplemented!();
        }
        AbortHandleType::NonEmulatableExit => {
            run.set_esr(esr_el2 & INST_ABORT_MASK);
            run.set_far(0);
            rmi::SUCCESS
        }
        _ => panic!("Shoudn't be reaching here"),
    };
    Ok(ret)
}
