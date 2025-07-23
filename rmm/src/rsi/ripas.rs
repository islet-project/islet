use crate::granule::is_granule_aligned;
use crate::granule::GranuleState;
use crate::realm::mm::stage2_tte::ripas;
use crate::realm::rd::Rd;
use crate::rec::context::{get_reg, set_reg};
use crate::rec::Rec;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmi::rec::run::{EntryFlag, Run};
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
    let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>()?;

    let base = get_reg(rec, 1)?;
    let top = get_reg(rec, 2)?;
    if !is_granule_aligned(base)
        || !is_granule_aligned(top)
        || !rd.addr_in_par(base)
        || !rd.addr_in_par(top - 1)
        || top <= base
    {
        if set_reg(rec, 0, rsi::ERROR_INPUT).is_err() {
            warn!("Unable to set register 0. rec: {:?}", rec);
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        return Ok(());
    }

    let res = crate::realm::mm::rtt::get_ripas(&rd, base, top);
    let (out_top, ripas) = if let Ok((out_top, ripas)) = res {
        if out_top > top {
            (top, ripas)
        } else {
            (out_top, ripas)
        }
    } else {
        if set_reg(rec, 0, rsi::ERROR_INPUT).is_err() {
            warn!("Unable to set register 0. rec: {:?}", rec);
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        return Ok(());
    };

    debug!(
        "RSI_IPA_STATE_GET: base: {:X} top: {:X} out_top: {:X} ripas: {:X}",
        base, top, out_top, ripas
    );

    if set_reg(rec, 0, rsi::SUCCESS).is_err() {
        warn!("Unable to set register 0. rec: {:?}", rec);
    }

    if set_reg(rec, 1, out_top).is_err() {
        warn!("Unable to set register 1. rec: {:?}", rec);
    }

    if set_reg(rec, 2, ripas as usize).is_err() {
        warn!("Unable to set register 2. rec: {:?}", rec);
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

    let rd_granule = get_granule_if!(rec.owner()?, GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>()?;

    if !is_granule_aligned(ipa_start)
        || !is_granule_aligned(ipa_end)
        || !is_ripas_valid(ipa_state)
        || ipa_end <= ipa_start
        || !rd.addr_in_par(ipa_start)
        || !rd.addr_in_par(ipa_end - 1)
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
        ripas::EMPTY | ripas::RAM => true,
        _ => false,
    }
}

pub fn complete_ripas(rec: &mut Rec<'_>, run: &Run) -> Result<(), Error> {
    let ripas_addr = rec.ripas_addr() as usize;
    if rec.ripas_end() as usize > 0 {
        set_reg(rec, 0, rsi::SUCCESS)?; // RSI_SUCCESS
        set_reg(rec, 1, ripas_addr)?;
        let flags = run.entry_flags();
        if flags.get_masked(EntryFlag::RIPAS_RESPONSE) != 0 {
            set_reg(rec, 2, 1)?; // REJECT
        } else {
            set_reg(rec, 2, 0)?; // ACCEPT
        }
        rec.set_ripas(0, 0, 0, 0);
    }
    Ok(())
}
