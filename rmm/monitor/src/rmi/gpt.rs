use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmm::granule;
use crate::rmm::granule::{GranuleState, RmmGranule};
use crate::rmm::PageMap;
use crate::smc;
extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::GRANULE_DELEGATE, |ctx, rmm| {
        let smc = rmm.smc;
        let mm = rmm.mm;
        ctx.ret = mark_realm(smc, mm, ctx.arg[0]);
    });

    listen!(mainloop, rmi::GRANULE_UNDELEGATE, |ctx, rmm| {
        let smc = rmm.smc;
        let mm = rmm.mm;
        ctx.ret = mark_ns(smc, mm, ctx.arg[0]);
    });
}

pub fn mark_realm(smc: smc::SecureMonitorCall, mm: PageMap, addr: usize) -> [usize; 8] {
    let cmd = smc.convert(smc::Code::MarkRealm);
    let arg = [addr, 0, 0, 0];
    let ret = smc.call(cmd, arg);
    if ret[0] == smc::SMC_SUCCESS {
        let g = granule::find_granule(addr, GranuleState::Undelegated).unwrap();
        g.set_state(GranuleState::Delegated, mm);
    }
    ret
}

pub fn mark_ns(smc: smc::SecureMonitorCall, mm: PageMap, addr: usize) -> [usize; 8] {
    let cmd = smc.convert(smc::Code::MarkNonSecure);
    let arg = [addr, 0, 0, 0];
    let ret = smc.call(cmd, arg);
    if ret[0] == smc::SMC_SUCCESS {
        let g = granule::find_granule(addr, GranuleState::Delegated).unwrap();
        g.set_state(GranuleState::Undelegated, mm);
    }
    ret
}
