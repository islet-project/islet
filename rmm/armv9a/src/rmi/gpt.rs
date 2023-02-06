use monitor::{smc, listen, mainloop::Mainloop, rmi};
use crate::config;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::GranuleDelegate, |call| {
        let cmd = usize::from(config::Code::MarkRealm);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        Ok(call.reply(ret[0])?)
    });

    listen!(mainloop, rmi::Code::GranuleUndelegate, |call| {
        let cmd = usize::from(config::Code::MarkNonSecure);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        Ok(call.reply(ret[0])?)
    });
}
