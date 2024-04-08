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

// [JB] for Cloak
use crate::rmi::rec::handlers::walk_page_table;
use crate::realm::registry::get_realm;

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
        CHANNEL_CREATE          = 0xc400_0200, // for Cloak
        CHANNEL_CONNECT         = 0xc400_0201, // for Cloak
        CHANNEL_GEN_REPORT      = 0xc400_0202, // for Cloak
        CHANNEL_RESULT          = 0xc400_0203, // for Cloak
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

use spin::mutex::Mutex;
use alloc::collections::btree_map::BTreeMap;

const CHANNEL_STATE_CREATED: usize = 0;
const CHANNEL_STATE_CONNECTED: usize = 1;
const CHANNEL_STATE_ESTABLISHED: usize = 2;

// LocalChannel for Cloak
#[allow(dead_code)]
#[derive(Copy, Clone)]
struct LocalChannel {
    id: usize,
    creator_registered: bool,
    connector_registered: bool,
    creator_realmid: usize,
    connector_realmid: usize,
    creator_ipa: usize,
    connector_ipa: usize,
    connector_token: [u8; 0x1000],
    state: usize,
}

impl LocalChannel {
    const fn create(id: usize, realmid: usize, ipa: usize) -> Self {
        let channel = LocalChannel {
            id: id,
            creator_registered: true,
            connector_registered: false,
            creator_realmid: realmid,
            connector_realmid: 0,
            creator_ipa: ipa,
            connector_ipa: 0,
            connector_token: [0; 0x1000],
            state: CHANNEL_STATE_CREATED,
        };
        channel
    }
}

static LOCAL_CHANNEL_TABLE: Mutex<BTreeMap<usize, LocalChannel>> = Mutex::new(BTreeMap::new());

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

fn do_attest_token_init(realmid: usize, vcpuid: usize, rec: &mut Rec<'_>) -> core::result::Result<(), Error> {
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
    Ok(())
}

fn write_token_to_ipa(realmid: usize, ipa_bits: usize, vcpuid: usize, attest_ipa: usize, token: &[u8; 4096]) -> core::result::Result<(), Error> {
    if validate_ipa(attest_ipa, ipa_bits).is_err() {
        warn!("Wrong ipa passed {}", attest_ipa);
        set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
        return Ok(());
    }

    let pa = crate::realm::registry::get_realm(realmid)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pa(GuestPhysAddr::from(attest_ipa), RTT_PAGE_LEVEL)
        .ok_or(Error::RmiErrorInput)?;
    let attest_pa: usize = pa.into();

    unsafe {
        let pa_ptr = attest_pa as *mut u8;
        core::ptr::copy(token.as_ptr(), pa_ptr, token.len());
    }
    Ok(())
}

fn do_attest_token_continue(realmid: usize, ipa_bits: usize, hash_algo: u8, vcpuid: usize, attest_ipa: usize, rec: &mut Rec<'_>) -> core::result::Result<(), Error> {
    if validate_ipa(attest_ipa, ipa_bits).is_err() {
        warn!("Wrong ipa passed {}", attest_ipa);
        set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
        return Ok(());
    }

    let pa = crate::realm::registry::get_realm(realmid)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pa(GuestPhysAddr::from(attest_ipa), RTT_PAGE_LEVEL)
        .ok_or(Error::RmiErrorInput)?;

    let mut measurements = crate::realm::registry::get_realm(realmid)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .measurements;

    // [JB] for cloak
    let mut rpv: [u8; 64] = [0; 64];
    match walk_page_table(realmid) {
        Ok(ns_count) => {
            let rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
            let rd = rd.content::<Rd>();
            let no_shared_region: usize = match rd.no_shared_region() {
                true => 1,
                false => 0,
            };
            let mut measurement: [u8; 64] = [0; 64];
            let _ = HashContext::new(&rd)?.read_measurement_with_input(no_shared_region, ns_count, &mut measurement, MEASUREMENTS_SLOT_RIM);
            measurements[MEASUREMENTS_SLOT_RIM].as_mut().copy_from_slice(&measurement);

            for (dst, src) in rpv.as_mut().iter_mut().zip(no_shared_region.to_ne_bytes().as_ref()) {
                *dst = *src;
            }
            info!("[WALK] ATTEST_TOKEN_CONTINUE: no_shared_region: {}, ns_count: {}, measure: {:x?}", no_shared_region, ns_count, measurement);
        },
        Err(_) => {},
    }

    let attest_size = crate::rsi::attestation::get_token(
        pa.into(),
        rec.attest_challenge(),
        &measurements,
        &rpv,
        hash_algo,
    );

    set_reg(realmid, vcpuid, 0, SUCCESS)?;
    set_reg(realmid, vcpuid, 1, attest_size)?;
    Ok(())
}

fn do_attest_token_init_channel(realmid: usize, vcpuid: usize, rec: &mut Rec<'_>) -> core::result::Result<(), Error> {
    let challenge: [u8; 64] = [0; 64];

    rec.set_attest_challenge(&challenge);
    rec.set_attest_state(RmmRecAttestState::AttestInProgress);

    set_reg(realmid, vcpuid, 0, SUCCESS)?;

    // TODO: Calculate real token size
    set_reg(realmid, vcpuid, 1, 4096)?;
    Ok(())
}

fn do_attest_token_continue_channel(realmid: usize, _ipa_bits: usize, hash_algo: u8, vcpuid: usize, out_token: &mut [u8; 0x1000], rec: &mut Rec<'_>) -> core::result::Result<(), Error> {
    let measurements = crate::realm::registry::get_realm(realmid)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .measurements;

    let rpv: [u8; 64] = [0; 64];
    let attest_size = crate::rsi::attestation::get_token_channel(
        out_token,
        rec.attest_challenge(),
        &measurements,
        &rpv,
        hash_algo,
    );

    set_reg(realmid, vcpuid, 0, SUCCESS)?;
    set_reg(realmid, vcpuid, 1, attest_size)?;
    Ok(())
}

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, ATTEST_TOKEN_INIT, |_arg, ret, _rmm, rec, _| {
        let realmid = rec.realmid()?;
        let vcpuid = rec.vcpuid();

        do_attest_token_init(realmid, vcpuid, rec)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
        /*
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
        Ok(()) */
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
        do_attest_token_continue(realmid, ipa_bits, hash_algo, vcpuid, attest_ipa, rec)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())

        /*
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

        let mut measurements = crate::realm::registry::get_realm(realmid)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .measurements;

        // [JB] for cloak
        let mut rpv: [u8; 64] = [0; 64];
        match walk_page_table(realmid) {
            Ok(ns_count) => {
                let rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
                let rd = rd.content::<Rd>();
                let no_shared_region: usize = match rd.no_shared_region() {
                    true => 1,
                    false => 0,
                };
                let mut measurement: [u8; 64] = [0; 64];
                let _ = HashContext::new(&rd)?.read_measurement_with_input(no_shared_region, ns_count, &mut measurement, MEASUREMENTS_SLOT_RIM);
                measurements[MEASUREMENTS_SLOT_RIM].as_mut().copy_from_slice(&measurement);

                for (dst, src) in rpv.as_mut().iter_mut().zip(no_shared_region.to_ne_bytes().as_ref()) {
                    *dst = *src;
                }
                info!("[WALK] ATTEST_TOKEN_CONTINUE: no_shared_region: {}, ns_count: {}, measure: {:x?}", no_shared_region, ns_count, measurement);
            },
            Err(_) => {},
        }

        let attest_size = crate::rsi::attestation::get_token(
            pa.into(),
            rec.attest_challenge(),
            &measurements,
            &rpv,
            hash_algo,
        );

        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        set_reg(realmid, vcpuid, 1, attest_size)?;

        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(()) */
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

        if index == MEASUREMENTS_SLOT_RIM {
            match walk_page_table(realmid) {
                Ok(ns_count) => {
                    let rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
                    let rd = rd.content::<Rd>();
                    let no_shared_region: usize = match rd.no_shared_region() {
                        true => 1,
                        false => 0,
                    };
                    let _ = HashContext::new(&rd)?.read_measurement_with_input(no_shared_region, ns_count, measurement.as_mut(), MEASUREMENTS_SLOT_RIM);
                    info!("[WALK] MEASUREMENT_READ: no_shared_region: {}, ns_count: {}, measure: {:x?}", no_shared_region, ns_count, measurement);
                },
                Err(_) => {},
            }
        } else {
            crate::rsi::measurement::read(realmid, index, &mut measurement)?;
        }

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

    listen!(rsi, CHANNEL_CREATE, |_arg, ret, _rmm, rec, _run| {
        let rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd.content::<Rd>();
        let realmid = rd.id();
        let vcpuid = rec.vcpuid();
        let channel_id = get_reg(realmid, vcpuid, 1)?;
        let ipa = get_reg(realmid, vcpuid, 2)?;

        let channel = LocalChannel::create(channel_id, realmid, ipa);
        LOCAL_CHANNEL_TABLE.lock().insert(channel_id, channel);

        info!("[JB] CHANNEL_CREATE success! realmid: {}, ipa: {:x}", realmid, ipa);
        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, CHANNEL_CONNECT, |_arg, ret, _rmm, rec, _run| {
        let rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd.content::<Rd>();
        let realmid = rd.id();
        let vcpuid = rec.vcpuid();
        let channel_id = get_reg(realmid, vcpuid, 1)?;
        let ipa = get_reg(realmid, vcpuid, 2)?;
        let ipa_bits = rec.ipa_bits()?;
        let hash_algo = rd.hash_algo();

        info!("CHANNEL_CONNECT start!");
        match LOCAL_CHANNEL_TABLE.lock().get_mut(&channel_id) {
            Some(channel) => {
                // 0. validation check
                // assert!(channel.connector_realmid != channel.creator_realmid)

                // 1. get a token, with nonce of zero
                info!("before do_attestation");
                do_attest_token_init_channel(realmid, vcpuid, rec)?;
                do_attest_token_continue_channel(realmid, ipa_bits, hash_algo, vcpuid, &mut channel.connector_token, rec)?;
                info!("after do_attestation");

                // 2. set a channel
                channel.connector_realmid = realmid;
                channel.connector_ipa = ipa;
                channel.creator_registered = true;
                channel.state = CHANNEL_STATE_CONNECTED;
                info!("[JB] CHANNEL_CONNECT success! realmid: {}, ipa: {:x}", realmid, ipa);

                // 3. create a shared mapping
                if channel.creator_realmid == channel.connector_realmid {
                    channel.state = CHANNEL_STATE_ESTABLISHED;
                    info!("[JB] test case! no shared mapping needed!");
                } else {
                    match create_shared_mapping(&channel) {
                        Ok(_) => info!("[JB] create_shared_mapping success"),
                        Err(_) => info!("[JB] create_shared_mapping fail"),
                    }
                    channel.state = CHANNEL_STATE_ESTABLISHED;
                    info!("[JB] CHANNEL_STATE_ESTABLISHED! creator_realmid: {}, creator_ipa: {:x}, connector_realmid: {}, connector_ipa: {:x}", channel.creator_realmid, channel.creator_ipa, channel.connector_realmid, channel.connector_ipa);
                }

                /*
                match walk_page_table(realmid) {
                    Ok(ns_count) => {
                        let no_shared_region: usize = match rd.no_shared_region() {
                            true => 1,
                            false => 0,
                        };
                        let mut measurement: [u8; 64] = [0; 64];
                        let _ = HashContext::new(&rd)?.read_measurement_with_input(no_shared_region, ns_count, &mut measurement, MEASUREMENTS_SLOT_RIM);
                        channel.runtime_connector_measurement.as_mut().copy_from_slice(&measurement);
                    },
                    Err(_) => {},
                }

                // 2. do mutual attestation
                info!("[JB] do mutual attestation here!");
                info!("[JB] expected_creator_measurement: {:x?}", channel.expected_creator_measurement);
                info!("[JB] runtime_creator_measurement: {:x?}", channel.runtime_creator_measurement);
                info!("[JB] expected_connector_measurement: {:x?}", channel.expected_connector_measurement);
                info!("[JB] runtime_connector_measurement: {:x?}", channel.runtime_connector_measurement);

                if channel.expected_creator_measurement == channel.runtime_creator_measurement &&
                    channel.expected_connector_measurement == channel.runtime_connector_measurement {
                    info!("[JB] mutual attestation success!");
                } else {
                    info!("[JB] mutual attestation fail!");
                }
            
                // 3. create a shared memory mapping into connector_ipa and creator_ipa
                match create_shared_mapping(&channel) {
                    Ok(_) => info!("[JB] create_shared_mapping success"),
                    Err(_) => info!("[JB] create_shared_mapping fail"),
                }
                channel.state = CHANNEL_STATE_ESTABLISHED; */
            },
            None => {},
        }
        
        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, CHANNEL_GEN_REPORT, |_arg, ret, _rmm, rec, _run| {
        let realmid = rec.realmid()?;
        let vcpuid = rec.vcpuid();
        let ipa_bits = rec.ipa_bits()?;
        let channel_id = get_reg(realmid, vcpuid, 1)?;
        let ipa = get_reg(realmid, vcpuid, 2)?;

        // input_r1: channel id
        // input_r2: ipa address that report is going to be written onto. 
        match LOCAL_CHANNEL_TABLE.lock().get_mut(&channel_id) {
            Some(channel) => {
                if channel.state != CHANNEL_STATE_ESTABLISHED {
                    info!("channel not yet established");
                    set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
                    return Ok(());
                }

                match write_token_to_ipa(realmid, ipa_bits, vcpuid, ipa, &channel.connector_token) {
                    Ok(_) => {},
                    Err(_) => {
                        info!("write_token_to_ipa error");
                        set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
                        return Ok(());
                    },
                }
            },
            None => {},
        }

        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, CHANNEL_RESULT, |_arg, ret, _rmm, rec, _run| {
        let realmid = rec.realmid()?;
        let vcpuid = rec.vcpuid();
        let channel_id = get_reg(realmid, vcpuid, 1)?;

        match LOCAL_CHANNEL_TABLE.lock().get(&channel_id) {
            Some(channel) => {
                if channel.state == CHANNEL_STATE_ESTABLISHED {
                    set_reg(realmid, vcpuid, 1, 1)?;  // connected: 1
                } else {
                    set_reg(realmid, vcpuid, 1, 0)?;  // not connected: 0
                }

                /*
                if channel_result == 1 {
                    match create_shared_mapping(&channel) {
                        Ok(_) => info!("[JB] create_shared_mapping success"),
                        Err(_) => info!("[JB] create_shared_mapping fail"),
                    }

                    channel.state = CHANNEL_STATE_ESTABLISHED;
                    info!("[JB] CHANNEL_STATE_ESTABLISHED! creator_realmid: {}, creator_ipa: {:x}, connector_realmid: {}, connector_ipa: {:x}", channel.creator_realmid, channel.creator_ipa, channel.connector_realmid, channel.connector_ipa);
                } */
            },
            None => {
                return Err(Error::RmiErrorInput);
            },
        }

        set_reg(realmid, vcpuid, 0, SUCCESS)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

}

fn is_ripas_valid(ripas: u8) -> bool {
    match ripas as u64 {
        invalid_ripas::EMPTY | invalid_ripas::RAM => true,
        _ => false,
    }
}

fn create_shared_mapping(channel: &LocalChannel) -> Result<(), Error> {
    // test:
    //   3-1: get pte of connector_ipa (pa)
    //   3-2: copy pte to creator's RTT
    //   3-3: two new syscalls for test (write_to_channel, read_from_channel)
    let res = get_realm(channel.connector_realmid)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte(GuestPhysAddr::from(channel.connector_ipa), RTT_PAGE_LEVEL);

    let connector_pte = if let Some(p) = res {
        p.0
    } else {
        info!("[JB] connector ipa_to_pte error");
        return Err(Error::RmiErrorInput);
    };
    info!("[JB] connector pte: {}", connector_pte);

    let res = get_realm(channel.creator_realmid)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(channel.creator_ipa), RTT_PAGE_LEVEL, connector_pte);

    match res {
        Ok(_) => {
            info!("[JB] creator ipa_to_pte_set success");
            Ok(())
        },
        Err(e) => Err(e),
    }
}
