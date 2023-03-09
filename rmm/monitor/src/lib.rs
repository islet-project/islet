#![no_std]
#![allow(incomplete_features)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

pub mod error;
pub mod io;
pub mod logger;
pub mod r#macro;
pub mod mm;
pub mod realm;
pub mod rmi;
pub mod smc;

pub(crate) mod event;
pub(crate) mod utils; // TODO: move to lib

#[macro_use]
extern crate log;

use crate::event::Mainloop;
use crate::rmi::RMI;
use crate::smc::SecureMonitorCall;

pub struct Monitor {
    mainloop: Mainloop,
    rmi: RMI,
    smc: SecureMonitorCall,
}

impl Monitor {
    pub fn new(rmi: RMI, smc: smc::SecureMonitorCall) -> Self {
        let mut mainloop = Mainloop::new();
        Self::add_event_handler(&mut mainloop);
        Self { mainloop, rmi, smc }
    }

    fn add_event_handler(mainloop: &mut Mainloop) {
        rmi::version::set_event_handler(mainloop);
        rmi::gpt::set_event_handler(mainloop);
        rmi::realm::set_event_handler(mainloop);
    }

    pub fn boot_complete(&self) {
        let ctx = event::Context {
            cmd: rmi::BOOT_COMPLETE,
            arg: [rmi::BOOT_SUCCESS, 0, 0, 0],
            ..Default::default()
        };

        self.mainloop.dispatch(self.smc, ctx);
    }

    pub fn run(&self) {
        self.mainloop.run(self.rmi, self.smc);
    }
}
