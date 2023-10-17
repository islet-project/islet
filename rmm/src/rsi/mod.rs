pub mod constraint;
pub mod error;
pub mod hostcall;
pub mod psci;

use crate::define_interface;
use crate::event::RsiHandle;
use crate::granule::GranuleState;
use crate::listen;
use crate::measurement::{
    HashContext, Measurement, MeasurementError, MEASUREMENTS_SLOT_NR, MEASUREMENTS_SLOT_RIM,
};
use crate::rmi;
use crate::rmi::error::{Error, InternalError::NotExistRealm};
use crate::rmi::realm::Rd;
use crate::rmi::rec::run::Run;
use crate::rmi::rec::Rec;
use crate::rmi::rtt::{validate_ipa, RTT_PAGE_LEVEL};
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
    let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
    let realmid = g_rd.content::<Rd>().id();
    drop(g_rd);

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
}

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, HOST_CALL, do_host_call);

    listen!(rsi, ABI_VERSION, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let realmid = g_rd.content::<Rd>().id();
        drop(g_rd);

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
        let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let realmid = g_rd.content::<Rd>().id();
        drop(g_rd);

        let mut measurement = Measurement::empty();
        let index = rmi.get_reg(realmid, vcpuid, 1)?;
        rmm.rsi.measurement_read(realmid, index, &mut measurement)?;
        rmi.set_reg(realmid, vcpuid, 0, SUCCESS)?;
        for (ind, chunk) in measurement
            .as_slice()
            .chunks_exact(core::mem::size_of::<usize>())
            .enumerate()
        {
            let reg_value = usize::from_le_bytes(chunk.try_into().unwrap());
            if rmi.set_reg(realmid, vcpuid, ind + 1, reg_value).is_err() {
                warn!(
                    "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                    realmid, vcpuid
                );
            }
        }

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, MEASUREMENT_EXTEND, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let vcpuid = rec.id();
        let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let realmid = g_rd.content::<Rd>().id();
        drop(g_rd);

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
            return Err(crate::event::Error::RmiErrorInput);
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
        let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let rd = g_rd.content::<Rd>();
        let ipa_bits = rd.ipa_bits();
        let realmid = rd.id();
        drop(g_rd);

        let config_ipa = rmi.get_reg(realmid, vcpuid, 1)?;
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
        let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let rd = g_rd.content::<Rd>();
        let ipa_bits = rd.ipa_bits();
        let realmid = rd.id();
        drop(g_rd);

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
        let g_rd = get_granule_if!(rec.owner(), GranuleState::RD)?;
        let realmid = g_rd.content::<Rd>().id();
        drop(g_rd);

        let ipa_start = rmi.get_reg(realmid, vcpuid, 1)? as u64;
        let ipa_size = rmi.get_reg(realmid, vcpuid, 2)? as u64;
        let ipa_state = rmi.get_reg(realmid, vcpuid, 3)? as u8;
        // TODO: check ipa_state value, ipa address granularity
        unsafe {
            run.set_exit_reason(rmi::EXIT_RIPAS_CHANGE);
            run.set_ripas(ipa_start, ipa_size, ipa_state);
            rec.set_ripas(ipa_start, ipa_start + ipa_size, ipa_start, ipa_state);
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
