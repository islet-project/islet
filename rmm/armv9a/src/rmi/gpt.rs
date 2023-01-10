use crate::rmi::Receiver;
use crate::smc;
use monitor::{listen, mainloop::Mainloop, rmi};

pub fn set_event_handler(mainloop: &mut Mainloop<Receiver>) {
    listen!(mainloop, rmi::Code::GranuleDelegate, |call| {
        let cmd = usize::from(smc::Code::MarkRealm);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        Ok(call.reply(ret[0])?)
    });

    listen!(mainloop, rmi::Code::GranuleUndelegate, |call| {
        let cmd = usize::from(smc::Code::MarkNonSecure);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        Ok(call.reply(ret[0])?)
    });
}
