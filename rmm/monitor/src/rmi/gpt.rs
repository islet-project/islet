use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::smc;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::GRANULE_DELEGATE, |ctx, _, smc, _| {
        ctx.ret = mark_realm(smc, ctx.arg[0]);
    });

    listen!(mainloop, rmi::GRANULE_UNDELEGATE, |ctx, _, smc, _| {
        ctx.ret = mark_ns(smc, ctx.arg[0]);
    });
}

pub fn mark_realm(smc: smc::SecureMonitorCall, addr: usize) -> [usize; 8] {
    let cmd = smc.convert(smc::Code::MarkRealm);
    let arg = [addr, 0, 0, 0];
    smc.call(cmd, arg)
}

pub fn mark_ns(smc: smc::SecureMonitorCall, addr: usize) -> [usize; 8] {
    let cmd = smc.convert(smc::Code::MarkNonSecure);
    let arg = [addr, 0, 0, 0];
    smc.call(cmd, arg)
}
