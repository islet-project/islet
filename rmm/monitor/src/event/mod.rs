pub mod mainloop;
pub mod realmexit;
pub mod rsihandle;

pub use mainloop::Mainloop;
pub use rsihandle::RsiHandle;

#[macro_export]
macro_rules! listen {
    ($eventloop:expr, $code:expr, $handler:expr) => {{
        $eventloop.add_event_handler($code.into(), alloc::boxed::Box::new($handler))
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
