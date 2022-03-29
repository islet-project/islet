use monitor::mainloop::Mainloop;

use armv9a::smc;

use crate::rmi;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    mainloop.set_event_handler(rmi::Code::GranuleDelegate, |call| {
        let cmd = usize::from(smc::Code::MarkRealm);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        let _ = call.reply(ret[0]);
    });

    mainloop.set_event_handler(rmi::Code::GranuleUndelegate, |call| {
        let cmd = usize::from(smc::Code::MarkNonSecure);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        let _ = call.reply(ret[0]);
    });
}
