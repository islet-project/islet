use monitor::{listen, mainloop::Mainloop};

use crate::config;
use crate::rmi;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::Version, |call| {
        trace!("RMM: requested version information");
        Ok(call.reply(config::ABI_VERSION)?)
    });
}
