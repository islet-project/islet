use crate::config;
use crate::error::{Error, ErrorKind};
use crate::listen;
use crate::mainloop::Mainloop;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::Version, |call| {
        let config = config::instance().ok_or(Error::new(ErrorKind::Unsupported))?;
        trace!("RMM: requested version information: {}", config.abi_version());
        Ok(call.reply(config.abi_version())?)
    });
}
