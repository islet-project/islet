use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::smc;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::Code::GranuleDelegate, |ctx| {
        let smc = smc::instance().unwrap();
        let cmd = smc.convert(smc::Code::MarkRealm);
        let arg = [ctx.arg[0], 0, 0, 0];
        ctx.ret = smc.call(cmd, arg);
    });

    listen!(mainloop, rmi::Code::GranuleUndelegate, |ctx| {
        let smc = smc::instance().unwrap();
        let cmd = smc.convert(smc::Code::MarkNonSecure);
        let arg = [ctx.arg[0], 0, 0, 0];
        ctx.ret = smc.call(cmd, arg);
    });
}
