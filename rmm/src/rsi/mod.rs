pub mod attestation;
pub mod constraint;
pub mod error;
pub mod hostcall;
pub mod measurement;
pub mod psci;

use crate::define_interface;
use crate::event::RsiHandle;
use crate::granule::{is_granule_aligned, GranuleState};
use crate::listen;
use crate::measurement::{HashContext, Measurement, MEASUREMENTS_SLOT_NR, MEASUREMENTS_SLOT_RIM};
use crate::realm::config::realm_config;
use crate::realm::context::{get_reg, set_reg};
use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::stage2_tte::invalid_ripas;
use crate::rmi;
use crate::rmi::error::{Error, InternalError::NotExistRealm};
use crate::rmi::realm::Rd;
use crate::rmi::rec::run::Run;
use crate::rmi::rec::{Rec, RmmRecAttestState};
use crate::rmi::rtt::{is_protected_ipa, validate_ipa, RTT_PAGE_LEVEL};
use crate::rsi::hostcall::{HostCall, HOST_CALL_NR_GPRS};
use crate::Monitor;
use crate::{get_granule, get_granule_if};

use safe_abstraction::raw_ptr::assume_safe;

define_interface! {
    command {
        ABI_VERSION             = 0xc400_0190,
        MEASUREMENT_READ        = 0xc400_0192,
        MEASUREMENT_EXTEND      = 0xc400_0193,
        ATTEST_TOKEN_INIT       = 0xc400_0194,
        ATTEST_TOKEN_CONTINUE   = 0xc400_0195,
        REALM_CONFIG            = 0xc400_0196,
        IPA_STATE_SET           = 0xc400_0197,
        IPA_STATE_GET           = 0xc400_0198,
        HOST_CALL               = 0xc400_0199,
    }
}

pub const SUCCESS: usize = 0;
pub const ERROR_INPUT: usize = 1;
pub const ERROR_STATE: usize = 2;
pub const INCOMPLETE: usize = 3;

const ABI_VERSION_MAJOR: usize = 1;
const ABI_VERSION_MINOR: usize = 0;
pub const VERSION: usize = (ABI_VERSION_MAJOR << 16) | ABI_VERSION_MINOR;

extern crate alloc;

pub fn do_host_call(
    _arg: &[usize],
    ret: &mut [usize],
    _rmm: &Monitor,
    rec: &mut Rec<'_>,
    run: &mut Run,
) -> core::result::Result<(), Error> {
    let vcpuid = rec.vcpuid();
    let realmid = rec.realmid()?;

    let ipa = get_reg(realmid, vcpuid, 1).unwrap_or(0x0);

    let pa = crate::realm::registry::get_realm(realmid)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pa(
            crate::realm::mm::address::GuestPhysAddr::from(ipa),
            RTT_PAGE_LEVEL,
        )
        .ok_or(Error::RmiErrorInput)?;

    let safety_assumed = assume_safe::<HostCall>(pa.into()).ok_or(Error::RmiErrorInput)?;
    let imm = safety_assumed.with(|host_call: &HostCall| host_call.imm());

    if rec.host_call_pending() {
        for i in 0..HOST_CALL_NR_GPRS {
            let val = run.entry_gpr(i)?;
            safety_assumed.mut_with(|host_call: &mut HostCall| host_call.set_gpr(i, val))?
        }
        rec.set_host_call_pending(false);
    } else {
        run.set_imm(imm);
        run.set_exit_reason(rmi::EXIT_HOST_CALL);
        rec.set_host_call_pending(true);
    }

    safety_assumed.with(|host_call: &HostCall| {
        trace!("HOST_CALL param: {:#X?}", host_call);
    });

    ret[0] = rmi::SUCCESS;
    Ok(())
}

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, ATTEST_TOKEN_INIT, |_arg, ret, _rmm, rec, _| {
        let realmid = rec.realmid()?;
        let vcpuid = rec.vcpuid();

        let mut challenge: [u8; 64] = [0; 64];

        for i in 0..8 {
            let challenge_part = get_reg(realmid, vcpuid, i + 2)?;
            let start_idx = i * 8;
            let end_idx = start_idx + 8;
            challenge[start_idx..end_idx].copy_from_slice(&challenge_part.to_le_bytes());
        }

        rec.set_attest_challenge(&challenge);
        rec.set_attest_state(RmmRecAttestState::AttestInProgress);

        set_reg(realmid, vcpuid, 0, SUCCESS)?;

        // TODO: Calculate real token size
        set_reg(realmid, vcpuid, 1, 4096)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, ATTEST_TOKEN_CONTINUE, |_arg, ret, _rmm, rec, _| {
        let realmid = rec.realmid()?;
        let ipa_bits = rec.ipa_bits()?;

        let hash_algo = get_granule_if!(rec.owner()?, GranuleState::RD)?
            .content::<Rd>()
            .hash_algo(); // Rd dropped

        let vcpuid = rec.vcpuid();

        if rec.attest_state() != RmmRecAttestState::AttestInProgress {
            warn!("Calling attest token continue without init");
            set_reg(realmid, vcpuid, 0, ERROR_STATE)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let attest_ipa = get_reg(realmid, vcpuid, 1)?;
        if validate_ipa(attest_ipa, ipa_bits).is_err() {
            warn!("Wrong ipa passed {}", attest_ipa);
            set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let pa = crate::realm::registry::get_realm(realmid)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pa(GuestPhysAddr::from(attest_ipa), RTT_PAGE_LEVEL)
            .ok_or(Error::RmiErrorInput)?;

        let measurements = crate::realm::registry::get_realm(realmid)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .measurements;

        let attest_size = crate::rsi::attestation::get_token(
            pa.into(),
            rec.attest_challenge(),
            &measurements,
            hash_algo,
        );

        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        set_reg(realmid, vcpuid, 1, attest_size)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, HOST_CALL, do_host_call);

    listen!(rsi, ABI_VERSION, |_arg, ret, _rmm, rec, _| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;

        if set_reg(realmid, vcpuid, 0, VERSION).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        trace!("RSI_ABI_VERSION: {:#X?}", VERSION);
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, MEASUREMENT_READ, |_arg, ret, _rmm, rec, _| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;
        let mut measurement = Measurement::empty();
        let index = get_reg(realmid, vcpuid, 1)?;

        if index >= MEASUREMENTS_SLOT_NR {
            warn!("Wrong index passed: {}", index);
            set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        crate::rsi::measurement::read(realmid, index, &mut measurement)?;
        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        for (ind, chunk) in measurement
            .as_slice()
            .chunks_exact(core::mem::size_of::<usize>())
            .enumerate()
        {
            let reg_value = usize::from_le_bytes(chunk.try_into().unwrap());
            set_reg(realmid, vcpuid, ind + 1, reg_value)?;
        }

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, MEASUREMENT_EXTEND, |_arg, ret, _rmm, rec, _| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;

        let index = get_reg(realmid, vcpuid, 1)?;
        let size = get_reg(realmid, vcpuid, 2)?;
        let mut buffer = [0u8; 64];

        for i in 0..8 {
            buffer[i * 8..i * 8 + 8]
                .copy_from_slice(get_reg(realmid, vcpuid, i + 3)?.to_le_bytes().as_slice());
        }

        if size > buffer.len() || index == MEASUREMENTS_SLOT_RIM || index >= MEASUREMENTS_SLOT_NR {
            warn!(
                "Wrong index or buffer size passed: idx: {}, size: {}",
                index, size
            );
            set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd.content::<Rd>();
        HashContext::new(rd)?.extend_measurement(&buffer[0..size], index)?;

        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, REALM_CONFIG, |_arg, ret, _rmm, rec, _| {
        let vcpuid = rec.vcpuid();
        let ipa_bits = rec.ipa_bits()?;
        let realmid = rec.realmid()?;
        let config_ipa = get_reg(realmid, vcpuid, 1)?;
        if validate_ipa(config_ipa, ipa_bits).is_err() {
            set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        realm_config(realmid, config_ipa, ipa_bits)?;

        if set_reg(realmid, vcpuid, 0, SUCCESS).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, IPA_STATE_GET, |_arg, ret, _rmm, rec, _| {
        let vcpuid = rec.vcpuid();
        let ipa_bits = rec.ipa_bits()?;
        let realmid = rec.realmid()?;

        let ipa_page = get_reg(realmid, vcpuid, 1)?;
        if validate_ipa(ipa_page, ipa_bits).is_err() {
            if set_reg(realmid, vcpuid, 0, ERROR_INPUT).is_err() {
                warn!(
                    "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                    realmid, vcpuid
                );
            }
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let ripas = crate::rtt::get_ripas(realmid, ipa_page, RTT_PAGE_LEVEL)? as usize;

        debug!(
            "RSI_IPA_STATE_GET: ipa_page: {:X} ripas: {:X}",
            ipa_page, ripas
        );

        if set_reg(realmid, vcpuid, 0, SUCCESS).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }

        if set_reg(realmid, vcpuid, 1, ripas).is_err() {
            warn!(
                "Unable to set register 1. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, IPA_STATE_SET, |_arg, ret, _rmm, rec, run| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;
        let ipa_bits = rec.ipa_bits()?;

        let ipa_start = get_reg(realmid, vcpuid, 1)?;
        let ipa_size = get_reg(realmid, vcpuid, 2)?;
        let ipa_state = get_reg(realmid, vcpuid, 3)? as u8;
        let ipa_end = ipa_start + ipa_size;

        if ipa_end <= ipa_start {
            return Err(Error::RmiErrorInput); // integer overflows or size is zero
        }

        if !is_granule_aligned(ipa_start)
            || !is_granule_aligned(ipa_size)
            || !is_ripas_valid(ipa_state)
            || !is_protected_ipa(ipa_start, ipa_bits)
            || !is_protected_ipa(ipa_end - 1, ipa_bits)
        {
            set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        // TODO: check ipa_state value, ipa address granularity
        run.set_exit_reason(rmi::EXIT_RIPAS_CHANGE);
        run.set_ripas(ipa_start as u64, ipa_size as u64, ipa_state);
        rec.set_ripas(
            ipa_start as u64,
            ipa_end as u64,
            ipa_start as u64,
            ipa_state,
        );
        ret[0] = rmi::SUCCESS;
        debug!(
            "RSI_IPA_STATE_SET: {:X} ~ {:X} {:X}",
            ipa_start,
            ipa_start + ipa_size,
            ipa_state
        );
        super::rmi::dummy();
        Ok(())
    });
}

fn is_ripas_valid(ripas: u8) -> bool {
    match ripas as u64 {
        invalid_ripas::EMPTY | invalid_ripas::RAM => true,
        _ => false,
    }
}
