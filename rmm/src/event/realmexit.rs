use crate::event::{Context, RsiHandle};
use crate::granule::{GranuleState, GRANULE_MASK};
use crate::realm::mm::stage2_tte::S2TTE;
use crate::rmi::error::Error;
use crate::rmi::realm::Rd;
use crate::rmi::rec::run::Run;
use crate::rmi::rec::Rec;
use crate::rmi::rtt::is_protected_ipa;
use crate::rmi::rtt::RTT_PAGE_LEVEL;
use crate::rmi::RMI;
use crate::Monitor;
use crate::{get_granule, get_granule_if};
use crate::{rmi, rsi};
use armv9a::{EsrEl2, EMULATABLE_ABORT_MASK, HPFAR_EL2, NON_EMULATABLE_ABORT_MASK};

const RMI_REC_EXIT_REASON_MASK: usize = 15; // 0b1111

const EXIT_SYNC_TYPE_SHIFT: usize = 4;
const EXIT_SYNC_TYPE_MASK: usize = 15 << EXIT_SYNC_TYPE_SHIFT; // 0b1111_0000

#[derive(Debug)]
#[repr(usize)]
pub enum RmiRecExitReason {
    Sync(ExitSyncType),
    IRQ = 1,
    FIQ = 2,
    PSCI = 3,
    RipasChange = 4,
    HostCall = 5,
    SError = 6,
    Undefined = RMI_REC_EXIT_REASON_MASK, // fixed, 0b1111
}

impl Into<u64> for RmiRecExitReason {
    fn into(self) -> u64 {
        match self {
            RmiRecExitReason::Sync(exit_sync_type) => exit_sync_type.into(),
            RmiRecExitReason::IRQ => 1,
            RmiRecExitReason::FIQ => 2,
            RmiRecExitReason::PSCI => 3,
            RmiRecExitReason::RipasChange => 4,
            RmiRecExitReason::HostCall => 5,
            RmiRecExitReason::SError => 6,
            RmiRecExitReason::Undefined => 7,
        }
    }
}

impl From<usize> for RmiRecExitReason {
    fn from(num: usize) -> Self {
        match num & RMI_REC_EXIT_REASON_MASK {
            0 => RmiRecExitReason::Sync(ExitSyncType::from(num)),
            1 => RmiRecExitReason::IRQ,
            2 => RmiRecExitReason::FIQ,
            3 => RmiRecExitReason::PSCI,
            4 => RmiRecExitReason::RipasChange,
            5 => RmiRecExitReason::HostCall,
            6 => RmiRecExitReason::SError,
            _ => RmiRecExitReason::Undefined,
        }
    }
}

/// Fault Status Code only for the 'RmiRecExitReason::Sync'
/// it's a different from trap::Syndrome
#[derive(Debug)]
#[repr(usize)]
pub enum ExitSyncType {
    RSI = 1 << EXIT_SYNC_TYPE_SHIFT,
    DataAbort = 2 << EXIT_SYNC_TYPE_SHIFT,
    InstAbort = 3 << EXIT_SYNC_TYPE_SHIFT,
    Undefined = EXIT_SYNC_TYPE_MASK, // fixed, 0b1111_0000
}

impl Into<u64> for ExitSyncType {
    fn into(self) -> u64 {
        self as u64
    }
}

impl From<usize> for ExitSyncType {
    fn from(num: usize) -> Self {
        let masked_val = (num & EXIT_SYNC_TYPE_MASK) >> EXIT_SYNC_TYPE_SHIFT;
        match masked_val {
            1 => ExitSyncType::RSI,
            2 => ExitSyncType::DataAbort,
            3 => ExitSyncType::InstAbort,
            _ => ExitSyncType::Undefined,
        }
    }
}

pub fn handle_realm_exit(
    realm_exit_res: [usize; 4],
    rmm: &Monitor,
    rec: &mut Rec,
    run: &mut Run,
) -> Result<(bool, usize), Error> {
    let mut return_to_ns = true;
    let ret = match RmiRecExitReason::from(realm_exit_res[0]) {
        RmiRecExitReason::Sync(ExitSyncType::RSI) => {
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
        RmiRecExitReason::Sync(ExitSyncType::DataAbort) => {
            handle_data_abort(realm_exit_res, rmm, rec, run)?
        }
        RmiRecExitReason::IRQ => unsafe {
            run.set_exit_reason(rmi::EXIT_IRQ);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            run.set_far(realm_exit_res[3] as u64);
            rmi::SUCCESS
        },
        RmiRecExitReason::Sync(ExitSyncType::InstAbort)
        | RmiRecExitReason::Sync(ExitSyncType::Undefined) => unsafe {
            run.set_exit_reason(rmi::EXIT_SYNC);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            run.set_far(realm_exit_res[3] as u64);
            rmi::SUCCESS
        },
        _ => rmi::SUCCESS,
    };

    Ok((return_to_ns, ret))
}

fn is_non_emulatable_data_abort(
    realm_id: usize,
    ipa_bits: usize,
    fault_ipa: usize,
    esr_el2: u64,
) -> Result<bool, Error> {
    let (s2tte, _) = S2TTE::get_s2tte(realm_id, fault_ipa, RTT_PAGE_LEVEL, Error::RmiErrorRtt(0))?;
    let is_protected_ipa = is_protected_ipa(fault_ipa, ipa_bits);

    let ret = match is_protected_ipa {
        true => s2tte.is_unassigned() || s2tte.is_destroyed(),
        false => (s2tte.is_unassigned() && (esr_el2 & EsrEl2::ISV) == 0) || s2tte.is_assigned(),
    };

    Ok(ret)
}

fn get_write_val(rmi: RMI, realm_id: usize, vcpu_id: usize, esr_el2: u64) -> Result<u64, Error> {
    let esr_el2 = EsrEl2::new(esr_el2);
    let rt = esr_el2.get_masked_value(EsrEl2::SRT) as usize;
    let write_val = match rt == 31 {
        true => 0, // xzr
        false => rmi.get_reg(realm_id, vcpu_id, rt)? as u64 & esr_el2.get_access_size_mask(),
    };
    Ok(write_val)
}

fn handle_data_abort(
    realm_exit_res: [usize; 4],
    rmm: &Monitor,
    rec: &mut Rec,
    run: &mut Run,
) -> Result<usize, Error> {
    let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
    let rd = g_rd.content::<Rd>();
    let realm_id = rd.id();
    let ipa_bits = rd.ipa_bits();
    drop(g_rd); // manually drop to reduce a lock contention

    let esr_el2 = realm_exit_res[1] as u64;
    let hpfar_el2 = realm_exit_res[2] as u64;
    let far_el2 = realm_exit_res[3] as u64;

    unsafe {
        run.set_exit_reason(rmi::EXIT_SYNC);
        run.set_hpfar(hpfar_el2);
    }

    let fault_ipa = ((HPFAR_EL2::FIPA & hpfar_el2) << 8) as usize;

    let (exit_esr, exit_far) =
        match is_non_emulatable_data_abort(realm_id, ipa_bits, fault_ipa, esr_el2)? {
            true => (esr_el2 & NON_EMULATABLE_ABORT_MASK, 0),
            false => {
                if esr_el2 & EsrEl2::WNR != 0 {
                    let write_val = get_write_val(rmm.rmi, realm_id, rec.id(), esr_el2)?;
                    unsafe {
                        run.set_gpr(0, write_val)?;
                    }
                }
                (
                    esr_el2 & EMULATABLE_ABORT_MASK,
                    (far_el2 & !(GRANULE_MASK as u64)),
                )
            }
        };

    unsafe {
        run.set_esr(exit_esr);
        run.set_far(exit_far);
    }

    Ok(rmi::SUCCESS)
}
