pub mod attestation;
pub mod constraint;
pub mod error;
pub mod hostcall;
pub mod measurement;
pub mod psci;
pub mod ripas;
pub mod version;

use alloc::vec::Vec;

use crate::define_interface;
use crate::event::RsiHandle;
use crate::granule::{GranuleState, GRANULE_SIZE};
use crate::listen;
use crate::measurement::{HashContext, Measurement, MEASUREMENTS_SLOT_NR, MEASUREMENTS_SLOT_RIM};
use crate::realm::config::realm_config;
use crate::realm::context::{get_reg, set_reg};
use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::rtt::RTT_PAGE_LEVEL;
use crate::realm::rd::Rd;
use crate::rec::{Rec, RmmRecAttestState};
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;
use crate::rmi::rtt::{is_protected_ipa, validate_ipa};
use crate::rsi::hostcall::{HostCall, HOST_CALL_NR_GPRS};
use crate::rsi::ripas::{get_ripas_state, set_ripas_state};
use crate::Monitor;
use crate::{get_granule, get_granule_if};

use safe_abstraction::raw_ptr::assume_safe;

define_interface! {
    command {
        ABI_VERSION             = 0xc400_0190,
        FEATURES                = 0xc400_0191,
        MEASUREMENT_READ        = 0xc400_0192,
        MEASUREMENT_EXTEND      = 0xc400_0193,
        ATTEST_TOKEN_INIT       = 0xc400_0194,
        ATTEST_TOKEN_CONTINUE   = 0xc400_0195,
        REALM_CONFIG            = 0xc400_0196,
        IPA_STATE_SET           = 0xc400_0197,
        IPA_STATE_GET           = 0xc400_0198,
        HOST_CALL               = 0xc400_0199,
        // PSCI smcs
        SMCCC_VERSION           = 0x8000_0000,
        PSCI_VERSION            = 0x8400_0000,
        PSCI_CPU_SUSPEND        = 0xC400_0001,
        PSCI_CPU_OFF            = 0x8400_0002,
        PSCI_CPU_ON             = 0xC400_0003,
        PSCI_AFFINITY_INFO      = 0xC400_0004,
        PSCI_SYSTEM_OFF         = 0x8400_0008,
        PSCI_SYSTEM_RESET       = 0x8400_0009,
        PSCI_FEATURES           = 0x8400_000A,
    }
}

pub const SUCCESS: usize = 0;
pub const ERROR_INPUT: usize = 1;
pub const ERROR_STATE: usize = 2;
pub const INCOMPLETE: usize = 3;

pub const ABI_VERSION_MAJOR: usize = 1;
pub const ABI_VERSION_MINOR: usize = 0;

extern crate alloc;

pub fn do_host_call(
    _arg: &[usize],
    ret: &mut [usize],
    _rmm: &Monitor,
    rec: &mut Rec<'_>,
    run: &mut Run,
) -> core::result::Result<(), Error> {
    let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>()?;

    let ipa = get_reg(rec, 1).unwrap_or(0x0);
    let ipa_bits = rec.ipa_bits()?;

    let struct_size = core::mem::size_of::<HostCall>();
    if ipa % struct_size != 0
        || ipa / GRANULE_SIZE != (ipa + struct_size - 1) / GRANULE_SIZE
        || !is_protected_ipa(ipa, ipa_bits)
    {
        set_reg(rec, 0, ERROR_INPUT)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        return Ok(());
    }

    let pa = rd
        .s2_table()
        .lock()
        .ipa_to_pa(
            crate::realm::mm::address::GuestPhysAddr::from(ipa),
            RTT_PAGE_LEVEL,
        )
        .ok_or(Error::RmiErrorInput)?;

    let mut host_call = assume_safe::<HostCall>(pa.into())?;
    let imm = host_call.imm();

    if rec.host_call_pending() {
        for i in 0..HOST_CALL_NR_GPRS {
            let val = run.entry_gpr(i)?;
            host_call.set_gpr(i, val)?
        }
        set_reg(rec, 0, SUCCESS)?;
        rec.set_host_call_pending(false);
    } else {
        for i in 0..HOST_CALL_NR_GPRS {
            let val = host_call.gpr(i)?;
            run.set_gpr(i, val)?
        }
        run.set_imm(imm);
        run.set_exit_reason(rmi::EXIT_HOST_CALL);
        rec.set_host_call_pending(true);
    }

    trace!("HOST_CALL param: {:#X?}", *host_call);

    ret[0] = rmi::SUCCESS;
    Ok(())
}

fn get_token_part(
    rd: &Rd,
    context: &mut Rec<'_>,
    size: usize,
) -> core::result::Result<(Vec<u8>, usize), Error> {
    let hash_algo = rd.hash_algo();
    let measurements = rd.measurements;

    // FIXME: This should be stored instead of generating it for every call.
    let token = crate::rsi::attestation::get_token(
        context.attest_challenge(),
        &measurements,
        rd.personalization_value(),
        hash_algo,
    );

    let offset = context.attest_token_offset();
    let part_size = core::cmp::min(size, token.len() - offset);
    let part_end = offset + part_size;

    context.set_attest_offset(part_end);

    Ok((token[offset..part_end].to_vec(), token.len() - part_end))
}

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, ATTEST_TOKEN_INIT, |_arg, ret, _rmm, rec, _| {
        let mut challenge: [u8; 64] = [0; 64];

        for i in 0..8 {
            let challenge_part = get_reg(rec, i + 1)?;
            let start_idx = i * 8;
            let end_idx = start_idx + 8;
            challenge[start_idx..end_idx].copy_from_slice(&challenge_part.to_le_bytes());
        }

        rec.set_attest_challenge(&challenge);
        rec.set_attest_state(RmmRecAttestState::AttestInProgress);
        rec.set_attest_offset(0);

        set_reg(rec, 0, SUCCESS)?;
        set_reg(rec, 1, attestation::MAX_CCA_TOKEN_SIZE)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, ATTEST_TOKEN_CONTINUE, |_arg, ret, _rmm, rec, _| {
        let ipa_bits = rec.ipa_bits()?;

        let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;

        if rec.attest_state() != RmmRecAttestState::AttestInProgress {
            warn!("Calling attest token continue without init");
            set_reg(rec, 0, ERROR_STATE)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let attest_ipa = get_reg(rec, 1)?;
        if validate_ipa(attest_ipa, ipa_bits).is_err() {
            warn!("Wrong ipa passed {}", attest_ipa);
            set_reg(rec, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let attest_pa: usize = rd
            .s2_table()
            .lock()
            .ipa_to_pa(GuestPhysAddr::from(attest_ipa), RTT_PAGE_LEVEL)
            .ok_or(Error::RmiErrorInput)?
            .into();

        let pa_offset = get_reg(rec, 2)?;
        let buffer_size = get_reg(rec, 3)?;

        let (_, overflowed) = pa_offset.overflowing_add(buffer_size);
        if overflowed || pa_offset + buffer_size > GRANULE_SIZE {
            warn!("Buffer addres region invalid");
            set_reg(rec, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        {
            let (token_part, token_left) = get_token_part(&rd, rec, buffer_size)?;

            unsafe {
                let pa_ptr = attest_pa as *mut u8;
                core::ptr::copy(token_part.as_ptr(), pa_ptr.add(pa_offset), token_part.len());
            }

            if token_left == 0 {
                set_reg(rec, 0, SUCCESS)?;
                rec.set_attest_state(RmmRecAttestState::NoAttestInProgress);
            } else {
                set_reg(rec, 0, INCOMPLETE)?;
            }

            set_reg(rec, 1, token_part.len())?;
        }

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, FEATURES, |_arg, ret, _rmm, rec, _| {
        let _index = get_reg(rec, 1);

        set_reg(rec, 0, SUCCESS)?;

        // B5.3.3 In the current version of the interface, this commands returns
        // zero regardless of the index provided.

        set_reg(rec, 1, 0)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, HOST_CALL, do_host_call);

    listen!(rsi, ABI_VERSION, |_arg, ret, _rmm, rec, _| {
        let req = get_reg(rec, 1)?;

        let (req_major, req_minor) = version::decode_version(req);

        if req_major != ABI_VERSION_MAJOR || req_minor != ABI_VERSION_MINOR {
            warn!(
                "Wrong unsupported version requested ({}, {})",
                req_major, req_minor
            );
            set_reg(rec, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        let lower = version::encode_version();
        let higher = lower;

        set_reg(rec, 0, SUCCESS)?;
        set_reg(rec, 1, lower)?;
        set_reg(rec, 2, higher)?;

        trace!("RSI_ABI_VERSION: {:#X?} {:#X?}", lower, higher);
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, MEASUREMENT_READ, |_arg, ret, _rmm, rec, _| {
        let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let mut measurement = Measurement::empty();
        let index = get_reg(rec, 1)?;

        if index >= MEASUREMENTS_SLOT_NR {
            warn!("Wrong index passed: {}", index);
            set_reg(rec, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        crate::rsi::measurement::read(&rd, index, &mut measurement)?;
        set_reg(rec, 0, SUCCESS)?;
        for (ind, chunk) in measurement
            .as_slice()
            .chunks_exact(core::mem::size_of::<usize>())
            .enumerate()
        {
            let reg_value = usize::from_le_bytes(chunk.try_into().unwrap());
            set_reg(rec, ind + 1, reg_value)?;
        }

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, MEASUREMENT_EXTEND, |_arg, ret, _rmm, rec, _| {
        let mut rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;

        let index = get_reg(rec, 1)?;
        let size = get_reg(rec, 2)?;
        let mut buffer = [0u8; 64];

        for i in 0..8 {
            buffer[i * 8..i * 8 + 8].copy_from_slice(get_reg(rec, i + 3)?.to_le_bytes().as_slice());
        }

        if size > buffer.len() || index == MEASUREMENTS_SLOT_RIM || index >= MEASUREMENTS_SLOT_NR {
            warn!(
                "Wrong index or buffer size passed: idx: {}, size: {}",
                index, size
            );
            set_reg(rec, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(&mut rd)?.extend_measurement(&buffer[0..size], index)?;

        set_reg(rec, 0, SUCCESS)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, REALM_CONFIG, |_arg, ret, _rmm, rec, _| {
        let ipa_bits = rec.ipa_bits()?;
        let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;

        let config_ipa = get_reg(rec, 1)?;
        if validate_ipa(config_ipa, ipa_bits).is_err() {
            set_reg(rec, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            return Ok(());
        }

        realm_config(&rd, config_ipa, ipa_bits)?;

        if set_reg(rec, 0, SUCCESS).is_err() {
            warn!("Unable to set register 0. rec: {:?}", rec);
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, IPA_STATE_GET, get_ripas_state);
    listen!(rsi, IPA_STATE_SET, set_ripas_state);
}
