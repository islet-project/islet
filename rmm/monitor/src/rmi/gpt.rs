use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmm::granule::{set_granule, GranuleState};
use crate::smc;
use crate::{get_granule, set_state_and_get_granule};
extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::GRANULE_DELEGATE, |arg, _, rmm| {
        mark_realm(rmm.smc, arg[0])
    });

    listen!(mainloop, rmi::GRANULE_UNDELEGATE, |arg, _, rmm| {
        mark_ns(rmm.smc, arg[0])
    });
}

pub fn mark_realm(smc: smc::SecureMonitorCall, addr: usize) -> Result<(), Error> {
    let mut granule = set_state_and_get_granule!(addr, GranuleState::Undelegated)?;

    let cmd = smc.convert(smc::Code::MarkRealm);
    if smc.call(cmd, &[addr])[0] != smc::SMC_SUCCESS {
        return Err(Error::RmiErrorInput);
    }

    set_granule(&mut granule, GranuleState::Delegated)?;
    Ok(())
}

pub fn mark_ns(smc: smc::SecureMonitorCall, addr: usize) -> Result<(), Error> {
    let mut granule = get_granule!(addr)?;
    if granule.state() != GranuleState::Delegated && granule.state() != GranuleState::Undelegated {
        return Err(Error::RmiErrorInput);
    }

    let cmd = smc.convert(smc::Code::MarkNonSecure);
    if smc.call(cmd, &[addr])[0] != smc::SMC_SUCCESS {
        panic!(
            "A delegated granule should only be undelegated on request from RMM. {:X}",
            addr
        );
    }

    set_granule(&mut granule, GranuleState::Undelegated)?;
    Ok(())
}
