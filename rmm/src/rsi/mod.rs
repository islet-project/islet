pub mod attestation;
pub mod constraint;
pub mod error;
pub mod hostcall;
pub mod psci;

use crate::define_interface;
use crate::event::RsiHandle;
use crate::granule::{is_granule_aligned, GranuleState};
use crate::listen;
use crate::measurement::{
    HashContext, Measurement, MeasurementError, MEASUREMENTS_SLOT_NR, MEASUREMENTS_SLOT_RIM,
};
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

pub const VERSION: usize = (1 << 16) | 0;

extern crate alloc;

pub fn do_host_call(
    _arg: &[usize],
    ret: &mut [usize],
    rmm: &Monitor,
    rec: &mut Rec,
    run: &mut Run,
) -> core::result::Result<(), Error> {
    let rmi = rmm.rmi;
    let vcpuid = rec.id();
    let realmid = rec.realmid();

    let ipa = rmi.get_reg(realmid, vcpuid, 1).unwrap_or(0x0);

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

    unsafe {
        let host_call = HostCall::parse_mut(pa.into());
        if rec.host_call_pending() {
            for i in 0..HOST_CALL_NR_GPRS {
                let val = run.entry_gpr(i)?;
                host_call.set_gpr(i, val)?
            }
            rec.set_host_call_pending(false);
        } else {
            run.set_imm(host_call.imm());
            run.set_exit_reason(rmi::EXIT_HOST_CALL);
            rec.set_host_call_pending(true);
        }
        trace!("HOST_CALL param: {:#X?}", host_call)
    }

    ret[0] = rmi::SUCCESS;
    Ok(())
}

pub trait Interface {
    fn measurement_read(
        &self,
        realmid: usize,
        index: usize,
        out: &mut Measurement,
    ) -> Result<(), error::Error>;
    fn measurement_extend(
        &self,
        realmid: usize,
        index: usize,
        f: impl Fn(&mut Measurement) -> Result<(), MeasurementError>,
    ) -> Result<(), error::Error>;
    fn get_attestation_token(
        &self,
        attest_pa: usize,
        challenge: &[u8],
        measurements: &[Measurement],
        hash_algo: u8,
    ) -> usize;
}

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, ATTEST_TOKEN_INIT, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let realmid = rec.realmid();
        let vcpuid = rec.id();

        let mut challenge: [u8; 64] = [0; 64];

        for i in 0..8 {
            let challenge_part = rmi.get_reg(realmid, vcpuid, i + 2)?;
            let start_idx = i * 8;
            let end_idx = start_idx + 8;
            challenge[start_idx..end_idx].copy_from_slice(&challenge_part.to_le_bytes());
        }

        rec.set_attest_challenge(&challenge);
        rec.set_attest_state(RmmRecAttestState::AttestInProgress);

        rmi.set_reg(realmid, vcpuid, 0, SUCCESS)?;

        // TODO: Calculate real token size
        rmi.set_reg(realmid, vcpuid, 1, 4096)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, ATTEST_TOKEN_CONTINUE, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;

        let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let rd = g_rd.content::<Rd>();
        let realmid = rd.id();
        let ipa_bits = rd.ipa_bits();
        let hash_algo = rd.hash_algo();
        drop(g_rd); // manually drop to reduce a lock contention

        let vcpuid = rec.id();

        if rec.attest_state() != RmmRecAttestState::AttestInProgress {
            warn!("Calling attest token continue without init");
            rmi.set_reg(realmid, vcpuid, 0, ERROR_STATE)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let attest_ipa = rmi.get_reg(realmid, vcpuid, 1)?;
        if validate_ipa(attest_ipa, ipa_bits).is_err() {
            warn!("Wrong ipa passed {}", attest_ipa);
            rmi.set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
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

        let attest_size = rmm.rsi.get_attestation_token(
            pa.into(),
            rec.attest_challenge(),
            &measurements,
            hash_algo,
        );

        rmi.set_reg(realmid, vcpuid, 0, SUCCESS)?;
        rmi.set_reg(realmid, vcpuid, 1, attest_size)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, HOST_CALL, do_host_call);

    listen!(rsi, ABI_VERSION, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let realmid = rec.realmid();

        if rmi.set_reg(realmid, vcpuid, 0, VERSION).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        trace!("RSI_ABI_VERSION: {:#X?}", VERSION);
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, MEASUREMENT_READ, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let realmid = rec.realmid();
        let mut measurement = Measurement::empty();
        let index = rmi.get_reg(realmid, vcpuid, 1)?;

        if index >= MEASUREMENTS_SLOT_NR {
            warn!("Wrong index passed: {}", index);
            rmi.set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        rmm.rsi.measurement_read(realmid, index, &mut measurement)?;
        rmi.set_reg(realmid, vcpuid, 0, SUCCESS)?;
        for (ind, chunk) in measurement
            .as_slice()
            .chunks_exact(core::mem::size_of::<usize>())
            .enumerate()
        {
            let reg_value = usize::from_le_bytes(chunk.try_into().unwrap());
            rmi.set_reg(realmid, vcpuid, ind + 1, reg_value)?;
        }

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, MEASUREMENT_EXTEND, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let realmid = rec.realmid();

        let index = rmi.get_reg(realmid, vcpuid, 1)?;
        let size = rmi.get_reg(realmid, vcpuid, 2)?;
        let mut buffer = [0u8; 64];

        for i in 0..8 {
            buffer[i * 8..i * 8 + 8].copy_from_slice(
                rmi.get_reg(realmid, vcpuid, i + 3)?
                    .to_le_bytes()
                    .as_slice(),
            );
        }

        if size > buffer.len() || index == MEASUREMENTS_SLOT_RIM || index >= MEASUREMENTS_SLOT_NR {
            warn!(
                "Wrong index or buffer size passed: idx: {}, size: {}",
                index, size
            );
            rmi.set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let rd = rd.content::<Rd>();
        HashContext::new(&rmm.rsi, &rd)?.extend_measurement(&buffer[0..size], index)?;

        rmi.set_reg(realmid, vcpuid, 0, SUCCESS)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, REALM_CONFIG, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let ipa_bits = rec.ipa_bits();
        let realmid = rec.realmid();
        let config_ipa = rmi.get_reg(realmid, vcpuid, 1)?;
        if validate_ipa(config_ipa, ipa_bits).is_err() {
            rmi.set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        rmi.realm_config(realmid, config_ipa, ipa_bits)?;

        if rmi.set_reg(realmid, vcpuid, 0, SUCCESS).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, IPA_STATE_GET, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let ipa_bits = rec.ipa_bits();
        let realmid = rec.realmid();

        let ipa_page = rmi.get_reg(realmid, vcpuid, 1)?;
        if validate_ipa(ipa_page, ipa_bits).is_err() {
            if rmi.set_reg(realmid, vcpuid, 0, ERROR_INPUT).is_err() {
                warn!(
                    "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                    realmid, vcpuid
                );
            }
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let ripas = rmi.rtt_get_ripas(realmid, ipa_page, RTT_PAGE_LEVEL)? as usize;

        debug!(
            "RSI_IPA_STATE_GET: ipa_page: {:X} ripas: {:X}",
            ipa_page, ripas
        );

        if rmi.set_reg(realmid, vcpuid, 0, SUCCESS).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }

        if rmi.set_reg(realmid, vcpuid, 1, ripas).is_err() {
            warn!(
                "Unable to set register 1. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, IPA_STATE_SET, |_arg, ret, rmm, rec, run| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let realmid = rec.realmid();
        let ipa_bits = rec.ipa_bits();

        let ipa_start = rmi.get_reg(realmid, vcpuid, 1)?;
        let ipa_size = rmi.get_reg(realmid, vcpuid, 2)?;
        let ipa_state = rmi.get_reg(realmid, vcpuid, 3)? as u8;
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
            rmi.set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        // TODO: check ipa_state value, ipa address granularity
        unsafe {
            run.set_exit_reason(rmi::EXIT_RIPAS_CHANGE);
            run.set_ripas(ipa_start as u64, ipa_size as u64, ipa_state);
            rec.set_ripas(
                ipa_start as u64,
                ipa_end as u64,
                ipa_start as u64,
                ipa_state,
            );
            ret[0] = rmi::SUCCESS;
        };
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
