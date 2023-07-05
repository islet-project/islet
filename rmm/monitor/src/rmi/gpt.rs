use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmm::granule;
use crate::rmm::granule::GranuleState;
use crate::rmm::PageMap;
use crate::smc;
extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::GRANULE_DELEGATE, |arg, ret, rmm| {
        let smc = rmm.smc;
        let mm = rmm.mm;
        mark_realm(smc, mm, arg[0], ret);
        Ok(())
    });

    listen!(mainloop, rmi::GRANULE_UNDELEGATE, |arg, ret, rmm| {
        let smc = rmm.smc;
        let mm = rmm.mm;
        mark_ns(smc, mm, arg[0], ret);
        Ok(())
    });
}

pub fn mark_realm(smc: smc::SecureMonitorCall, mm: PageMap, addr: usize, ret: &mut [usize]) {
    let cmd = smc.convert(smc::Code::MarkRealm);

    if granule::set_granule(addr, GranuleState::Delegated, mm) != granule::RET_SUCCESS {
        ret[0] = rmi::ERROR_INPUT;
        //ret[1] = addr; // [JB] ??
    } else {
        let smc_ret = smc.call(cmd, &[addr]);
        ret[0] = smc_ret[0];
    }
}

pub fn mark_ns(smc: smc::SecureMonitorCall, mm: PageMap, addr: usize, ret: &mut [usize]) {
    let cmd = smc.convert(smc::Code::MarkNonSecure);

    if granule::set_granule(addr, GranuleState::Undelegated, mm) != granule::RET_SUCCESS {
        ret[0] = rmi::ERROR_INPUT;
        // ret[1] = addr;  // [JB] GRANULE_DELEGATE & GRANULE_UNDELEGATE have only one output value
    } else {
        let smc_ret = smc.call(cmd, &[addr]);
        ret[0] = smc_ret[0];
    }
}
