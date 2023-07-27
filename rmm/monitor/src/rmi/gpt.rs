use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmm::granule;
use crate::rmm::granule::GranuleState;
use crate::smc;
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
    let cmd = smc.convert(smc::Code::MarkRealm);
    if smc.call(cmd, &[addr])[0] != smc::SMC_SUCCESS {
        return Err(Error::RmiErrorInput);
    }

    if granule::set_granule(addr, GranuleState::Delegated) != granule::RET_SUCCESS {
        return Err(Error::RmiErrorInput);
    }

    Ok(())
}

pub fn mark_ns(smc: smc::SecureMonitorCall, addr: usize) -> Result<(), Error> {
    let cmd = smc.convert(smc::Code::MarkNonSecure);
    if smc.call(cmd, &[addr])[0] != smc::SMC_SUCCESS {
        panic!("A delegated granule should only be undelegated on request from RMM.");
    }

    if granule::set_granule(addr, GranuleState::Undelegated) != granule::RET_SUCCESS {
        return Err(Error::RmiErrorInput);
    }

    Ok(())
}
