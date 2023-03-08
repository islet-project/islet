use crate::listen;
use crate::mainloop::Mainloop;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    listen!(mainloop, rmi::Code::Version, |call| {
        trace!("RMM: requested version information: {}", rmi::ABI_VERSION);
        call.reply(&[rmi::ABI_VERSION, 0, 0, 0]);
        Ok(())
    });
}
