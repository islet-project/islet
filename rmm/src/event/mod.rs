pub mod mainloop;
pub mod realmexit;
pub mod rsihandle;

pub use crate::rmi::error::Error;
pub use crate::{rmi, rsi};
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

#[derive(Clone)]
pub struct Context {
    pub cmd: Command,
    pub arg: Vec<usize>,
    pub ret: Vec<usize>,
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

    pub fn do_rmi<F>(&mut self, handler: F) -> [usize; 5]
    where
        F: Fn(&[usize], &mut [usize]) -> Result<(), Error>,
    {
        let mut result = [0; 5];
        self.ret[0] = rmi::SUCCESS;

        #[cfg(feature = "stat")]
        {
            if self.cmd != rmi::REC_ENTER {
                trace!("let's get STATS.lock() with cmd {}", rmi::to_str(self.cmd));
                crate::stat::STATS.lock().measure(self.cmd, || {
                    if let Err(code) = handler(&self.arg[..], &mut self.ret[..]) {
                        self.ret[0] = code.into();
                    }
                });
            } else if let Err(code) = handler(&self.arg[..], &mut self.ret[..]) {
                self.ret[0] = code.into();
            }
        }
        #[cfg(not(feature = "stat"))]
        {
            if let Err(code) = handler(&self.arg[..], &mut self.ret[..]) {
                self.ret[0] = code.into();
            }
        }

        trace!(
            "RMI: {0: <20} {1:X?} > {2:X?}",
            rmi::to_str(self.cmd),
            &self.arg,
            &self.ret
        );

        self.arg.clear();
        self.arg.extend_from_slice(&self.ret[..]);

        let ret_len = self.ret.len();
        #[cfg(kani)]
        // the below is a proof helper
        {
            #[cfg(any(
                feature = "mc_rmi_granule_delegate",
                feature = "mc_rmi_granule_undelegate",
                feature = "mc_rmi_realm_activate",
                feature = "mc_rmi_realm_destroy",
                feature = "mc_rmi_rec_destroy"
            ))]
            assert!(ret_len == 1);
            #[cfg(any(feature = "mc_rmi_rec_aux_count", feature = "mc_rmi_features"))]
            assert!(ret_len == 2);
            #[cfg(feature = "mc_rmi_version")]
            assert!(ret_len == 3);
        }
        result[..ret_len].copy_from_slice(&self.ret[..]);
        result
    }

    pub fn do_rsi<F>(&mut self, mut handler: F)
    where
        F: FnMut(&[usize], &mut [usize]) -> Result<(), Error>,
    {
        self.ret[0] = rsi::SUCCESS;

        #[cfg(feature = "stat")]
        {
            trace!("let's get STATS.lock() with cmd {}", rsi::to_str(self.cmd));
            crate::stat::STATS.lock().measure(self.cmd, || {
                if let Err(code) = handler(&self.arg[..], &mut self.ret[..]) {
                    error!("rsi handler returns error:{:?}", code);
                    self.ret[0] = code.into();
                }
            });
        }
        #[cfg(not(feature = "stat"))]
        {
            if let Err(code) = handler(&self.arg[..], &mut self.ret[..]) {
                error!("rsi handler returns error:{:?}", code);
                self.ret[0] = code.into();
            }
        }

        trace!(
            "RSI: {0: <20} {1:X?} > {2:X?}",
            rsi::to_str(self.cmd),
            &self.arg,
            &self.ret
        );
        self.arg.clear();
        self.arg.extend_from_slice(&self.ret[..]);
    }
}

impl Default for Context {
    fn default() -> Context {
        Context::new(0)
    }
}
