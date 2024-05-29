use crate::granule::is_granule_aligned;
use crate::granule::GranuleState;
use crate::realm::context::{get_reg, set_reg};
use crate::realm::mm::stage2_tte::invalid_ripas;
use crate::realm::rd::Rd;
use crate::rec::Rec;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmi::rec::run::{Run, REC_ENTRY_FLAG_RIPAS_RESPONSE};
use crate::rmi::rtt::{is_protected_ipa, validate_ipa, RTT_PAGE_LEVEL};
use crate::rsi;
use crate::Monitor;
use crate::{get_granule, get_granule_if};

pub fn get_ripas_state(
    _arg: &[usize],
    ret: &mut [usize],
    _rmm: &Monitor,
    rec: &mut Rec<'_>,
    _run: &mut Run,
) -> core::result::Result<(), Error> {
    let ipa_bits = rec.ipa_bits()?;
    let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>();

    let ipa_page = get_reg(rec, 1)?;
    if validate_ipa(ipa_page, ipa_bits).is_err() {
        if set_reg(rec, 0, rsi::ERROR_INPUT).is_err() {
            warn!("Unable to set register 0. rec: {:?}", rec);
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        return Ok(());
    }

    let ripas = crate::rtt::get_ripas(rd, ipa_page, RTT_PAGE_LEVEL)? as usize;

    debug!(
        "RSI_IPA_STATE_GET: ipa_page: {:X} ripas: {:X}",
        ipa_page, ripas
    );

    if set_reg(rec, 0, rsi::SUCCESS).is_err() {
        warn!("Unable to set register 0. rec: {:?}", rec);
    }

    if set_reg(rec, 1, ripas).is_err() {
        warn!("Unable to set register 1. rec: {:?}", rec);
    }

    ret[0] = rmi::SUCCESS_REC_ENTER;
    Ok(())
}

pub fn set_ripas_state(
    _arg: &[usize],
    ret: &mut [usize],
    _rmm: &Monitor,
    rec: &mut Rec<'_>,
    run: &mut Run,
) -> core::result::Result<(), Error> {
    let ipa_bits = rec.ipa_bits()?;

    let ipa_start = get_reg(rec, 1)?;
    let ipa_end = get_reg(rec, 2)?;
    let ipa_state = get_reg(rec, 3)? as u8;
    let flags = get_reg(rec, 4)? as u64;

    if ipa_end <= ipa_start {
        set_reg(rec, 0, rsi::ERROR_INPUT)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        return Ok(());
        //return Err(Error::RmiErrorInput); // integer overflows or size is zero
    }

    if !is_granule_aligned(ipa_start)
        || !is_granule_aligned(ipa_end)
        || !is_ripas_valid(ipa_state)
        || ipa_end <= ipa_start
        || !is_protected_ipa(ipa_start, ipa_bits)
        || !is_protected_ipa(ipa_end - 1, ipa_bits)
    {
        set_reg(rec, 0, rsi::ERROR_INPUT)?;
        ret[0] = rmi::SUCCESS_REC_ENTER;
        return Ok(());
    }

    // TODO: check ipa_state value, ipa address granularity
    run.set_exit_reason(rmi::EXIT_RIPAS_CHANGE);
    run.set_ripas(ipa_start as u64, ipa_end as u64, ipa_state);
    rec.set_ripas(ipa_start as u64, ipa_end as u64, ipa_state, flags);
    ret[0] = rmi::SUCCESS;
    debug!(
        "RSI_IPA_STATE_SET: {:X} ~ {:X} {:X} {:X}",
        ipa_start, ipa_end, ipa_state, flags
    );
    Ok(())
}

fn is_ripas_valid(ripas: u8) -> bool {
    match ripas as u64 {
        invalid_ripas::EMPTY | invalid_ripas::RAM => true,
        _ => false,
    }
}

pub fn complete_ripas(rec: &mut Rec<'_>, run: &Run) -> Result<(), Error> {
    let ripas_addr = rec.ripas_addr() as usize;
    if rec.ripas_end() as usize > 0 {
        set_reg(rec, 0, rsi::SUCCESS)?; // RSI_SUCCESS
        set_reg(rec, 1, ripas_addr)?;
        let flags = run.entry_flags();
        if flags & REC_ENTRY_FLAG_RIPAS_RESPONSE != 0 {
            set_reg(rec, 2, 1)?; // REJECT
        } else {
            set_reg(rec, 2, 0)?; // ACCEPT
        }
        rec.set_ripas(0, 0, 0, 0);
    }
    Ok(())
}
