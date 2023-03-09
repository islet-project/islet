use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::VERSION, |ctx, _, _| {
        trace!("RMM: requested version information: {}", rmi::ABI_VERSION);
        ctx.ret[0] = rmi::ABI_VERSION;
    });
}
