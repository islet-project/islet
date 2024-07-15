use crate::event::realmexit::*;
use crate::event::{Context, RsiHandle};
use crate::get_granule;
use crate::get_granule_if;
use crate::granule::GranuleState;
use crate::granule::GRANULE_MASK;
use crate::realm::context::get_reg;
use crate::realm::mm::rtt::RTT_PAGE_LEVEL;
use crate::realm::mm::stage2_tte::S2TTE;
use crate::realm::rd::Rd;
use crate::rec::Rec;
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;
use crate::rmi::rtt::is_protected_ipa;
use crate::Monitor;
use crate::{rmi, rsi};
use armv9a::{EsrEl2, EMULATABLE_ABORT_MASK, NON_EMULATABLE_ABORT_MASK};

use aarch64_cpu::registers::HPFAR_EL2;

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
            let rsi = &rmm.rsi;
            let cmd = realm_exit_res[1];
            let mut ret = rmi::SUCCESS;

            rsi::constraint::validate(cmd, |_, ret_num| {
                let mut rsi_ctx = Context::new(cmd);
                rsi_ctx.resize_ret(ret_num);

                // set default value
                if rsi.dispatch(&mut rsi_ctx, rmm, rec, run) == RsiHandle::RET_SUCCESS {
                    if rsi_ctx.ret_slice()[0] == rmi::SUCCESS_REC_ENTER {
                        return_to_ns = false;
                    }
                    ret = rsi_ctx.ret_slice()[0];
                } else {
                    return_to_ns = false;
                }
            });
            ret
        }
        RecExitReason::Sync(ExitSyncType::DataAbort) => {
            handle_data_abort(realm_exit_res, rec, run)?
        }
        RecExitReason::IRQ => {
            run.set_exit_reason(rmi::EXIT_IRQ);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            run.set_far(realm_exit_res[3] as u64);
            rmi::SUCCESS
        }
        RecExitReason::Sync(ExitSyncType::InstAbort)
        | RecExitReason::Sync(ExitSyncType::Undefined) => {
            run.set_exit_reason(rmi::EXIT_SYNC);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            rmi::SUCCESS
        }
        _ => rmi::SUCCESS,
    };

    Ok((return_to_ns, ret))
}

fn is_non_emulatable_data_abort(
    rd: &Rd,
    ipa_bits: usize,
    fault_ipa: usize,
    esr_el2: u64,
) -> Result<bool, Error> {
    let (s2tte, _) = S2TTE::get_s2tte(rd, fault_ipa, RTT_PAGE_LEVEL, Error::RmiErrorRtt(0))?;
    let is_protected_ipa = is_protected_ipa(fault_ipa, ipa_bits);

    let ret = match is_protected_ipa {
        true => s2tte.is_unassigned() || s2tte.is_destroyed(),
        false => (s2tte.is_unassigned() && (esr_el2 & EsrEl2::ISV) == 0) || s2tte.is_assigned(),
    };

    Ok(ret)
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

fn handle_data_abort(
    realm_exit_res: [usize; 4],
    rec: &Rec<'_>,
    run: &mut Run,
) -> Result<usize, Error> {
    let ipa_bits = rec.ipa_bits()?;
    let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>()?;

    let esr_el2 = realm_exit_res[1] as u64;
    let hpfar_el2 = realm_exit_res[2] as u64;
    let far_el2 = realm_exit_res[3] as u64;

    run.set_exit_reason(rmi::EXIT_SYNC);
    run.set_hpfar(hpfar_el2);

    let fault_ipa = hpfar_el2 & (HPFAR_EL2::FIPA.mask << HPFAR_EL2::FIPA.shift);
    let fault_ipa = (fault_ipa << 8) as usize;

    let (exit_esr, exit_far) =
        match is_non_emulatable_data_abort(&rd, ipa_bits, fault_ipa, esr_el2)? {
            true => (esr_el2 & NON_EMULATABLE_ABORT_MASK, 0),
            false => {
                if esr_el2 & EsrEl2::WNR != 0 {
                    let write_val = get_write_val(rec, esr_el2)?;
                    run.set_gpr(0, write_val)?;
                }
                (
                    esr_el2 & EMULATABLE_ABORT_MASK,
                    (far_el2 & !(GRANULE_MASK as u64)),
                )
            }
        };

    run.set_esr(exit_esr);
    run.set_far(exit_far);

    Ok(rmi::SUCCESS)
}
