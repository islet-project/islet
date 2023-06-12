use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

fn encode_version() -> usize {
    (rmi::ABI_MAJOR_VERSION << 16) | rmi::ABI_MINOR_VERSION
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::VERSION, |_, ret, _| {
        ret[0] = encode_version();
    });
}
