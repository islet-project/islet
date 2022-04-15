use monitor::io::Write;
use monitor::mainloop::Mainloop;
use monitor::{eprintln, println};

use crate::config;
use crate::rmi;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    mainloop.set_event_handler(rmi::Code::Version, |call| {
        println!("RMM: requested version information");
        call.reply(config::ABI_VERSION)
            .err()
            .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });
}
