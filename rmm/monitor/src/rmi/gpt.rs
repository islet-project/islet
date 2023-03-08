use crate::error::{Error, ErrorKind};
use crate::listen;
use crate::mainloop::Mainloop;
use crate::rmi;
use crate::smc;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::GranuleDelegate, |call| {
        let smc = smc::instance().ok_or(Error::new(ErrorKind::Unsupported))?;
        let cmd = smc.convert(smc::Code::MarkRealm);
        let arg = [call.argument()[1], 0, 0, 0];
        let ret = smc.call(cmd, arg);
        call.reply(&[ret[0], ret[1], ret[2], ret[3]]);
        Ok(())
    });

    listen!(mainloop, rmi::Code::GranuleUndelegate, |call| {
        let smc = smc::instance().ok_or(Error::new(ErrorKind::Unsupported))?;
        let cmd = smc.convert(smc::Code::MarkNonSecure);
        let arg = [call.argument()[1], 0, 0, 0];
        let ret = smc.call(cmd, arg);
        call.reply(&[ret[0], ret[1], ret[2], ret[3]]);
        Ok(())
    });
}
