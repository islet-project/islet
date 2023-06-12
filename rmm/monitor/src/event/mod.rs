pub mod mainloop;
pub mod realmexit;
pub mod rsihandle;

pub use mainloop::Mainloop;
pub use rsihandle::RsiHandle;

extern crate alloc;
use alloc::vec::Vec;

#[macro_export]
macro_rules! listen {
    ($eventloop:expr, $code:expr, $handler:expr) => {{
        $eventloop.add_event_handler($code.into(), alloc::boxed::Box::new($handler))
    }};
}

pub type Command = usize;
pub type Argument = [usize; 8];
pub type Return = [usize; 8];

#[derive(Clone)]
pub struct Context {
    cmd: Command,
    arg: Vec<usize>,
    ret: Vec<usize>,
}

impl Context {
    pub fn new(cmd: Command) -> Context {
        Context {
            cmd,
            arg: Vec::new(),
            ret: Vec::new(),
        }
    }

    pub fn init_arg(&mut self, arg: &[usize]) {
        self.arg.clear();
        self.arg.extend_from_slice(arg);
    }

    pub fn init_ret(&mut self, ret: &[usize]) {
        self.ret.clear();
        self.ret.extend_from_slice(ret);
    }

    pub fn resize_ret(&mut self, new_len: usize) {
        self.ret.clear();
        self.ret.resize(new_len, 0);
    }

    pub fn arg_slice(&self) -> &[usize] {
        &self.arg[..]
    }

    pub fn ret_slice(&self) -> &[usize] {
        &self.ret[..]
    }

    pub fn cmd(&self) -> Command {
        self.cmd
    }

    pub fn do_rmi<F>(&mut self, handler: F)
    where
        F: Fn(&[usize], &mut [usize]),
    {
        handler(&self.arg[..], &mut self.ret[..]);
        self.arg.clear();
        self.arg.extend_from_slice(&self.ret[..]);
    }

    pub fn do_rsi<F>(&mut self, mut handler: F)
    where
        F: FnMut(&[usize], &mut [usize]),
    {
        handler(&self.arg[..], &mut self.ret[..]);
        self.arg.clear();
        self.arg.extend_from_slice(&self.ret[..]);
    }
}

impl Default for Context {
    fn default() -> Context {
        Context::new(0)
    }
}
