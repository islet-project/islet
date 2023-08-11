use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmm::granule::{set_granule, GranuleState};
use crate::rmm::PageMap;
use crate::smc;
use crate::{get_granule, get_granule_if, get_granule_not_if, set_state_and_get_granule};
extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::GRANULE_DELEGATE, |arg, _, rmm| {
        mark_realm(rmm.smc, rmm.mm, arg[0])
    });

    listen!(mainloop, rmi::GRANULE_UNDELEGATE, |arg, _, rmm| {
        mark_ns(rmm.smc, rmm.mm, arg[0])
    });
}

fn check_state_not_if(addr: usize, state: u64) -> Result<(), Error> {
    get_granule_not_if!(addr, state)?;
    Ok(())
}

fn check_state_if(addr: usize, state: u64) -> Result<(), Error> {
    get_granule_if!(addr, state)?;
    Ok(())
}

pub fn mark_realm(smc: smc::SecureMonitorCall, mm: PageMap, addr: usize) -> Result<(), Error> {
    if check_state_not_if(addr, GranuleState::Undelegated).is_ok() {
        // returns error if this is not in the state of Undelegated.
        return Err(Error::RmiErrorInput);
    }
    let mut granule = set_state_and_get_granule!(addr, GranuleState::Undelegated)?;

    let cmd = smc.convert(smc::Code::MarkRealm);
    if smc.call(cmd, &[addr])[0] != smc::SMC_SUCCESS {
        return Err(Error::RmiErrorInput);
    }

    mm.map(addr, true);
    set_granule(&mut granule, GranuleState::Delegated).map_err(|e| {
        mm.unmap(addr);
        e
    })?;
    mm.unmap(addr);
    Ok(())
}

pub fn mark_ns(smc: smc::SecureMonitorCall, mm: PageMap, addr: usize) -> Result<(), Error> {
    if check_state_if(addr, GranuleState::Undelegated).is_ok() {
        return Ok(());
    }
    let mut granule = get_granule_if!(addr, GranuleState::Delegated)?;

    let cmd = smc.convert(smc::Code::MarkNonSecure);
    if smc.call(cmd, &[addr])[0] != smc::SMC_SUCCESS {
        panic!(
            "A delegated granule should only be undelegated on request from RMM. {:X}",
            addr
        );
    }

    mm.map(addr, false);
    set_granule(&mut granule, GranuleState::Undelegated).map_err(|e| {
        mm.unmap(addr);
        e
    })?;
    mm.unmap(addr);
    Ok(())
}
