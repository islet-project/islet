mod mainloop;

extern crate alloc;

pub use mainloop::Mainloop;

use crate::realm::Manager;
use crate::smc::SecureMonitorCall;

use alloc::boxed::Box;

#[macro_export]
macro_rules! listen {
    ($mainloop:expr, $code:expr, $handler:expr) => {{
        $mainloop.add_event_handler($code.into(), alloc::boxed::Box::new($handler))
    }};
}

type Command = usize;
type Argument = [usize; 4];
type Return = [usize; 8];

#[derive(Default, Copy, Clone)]
pub struct Context {
    pub cmd: Command,
    pub arg: Argument,
    pub ret: Return,
}

pub type Handler = Box<dyn Fn(&mut Context, Manager, SecureMonitorCall)>;
