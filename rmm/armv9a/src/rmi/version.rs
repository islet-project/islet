use monitor::{listen, mainloop::Mainloop, rmi};

use crate::config;
use crate::rmi::Receiver;

pub fn set_event_handler(mainloop: &mut Mainloop<Receiver>) {
    listen!(mainloop, rmi::Code::Version, |call| {
        trace!("RMM: requested version information");
        Ok(call.reply(config::ABI_VERSION)?)
    });
}
