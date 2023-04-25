#![no_std]
#![allow(incomplete_features)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

pub mod error;
pub mod event;
pub mod io;
pub mod logger;
pub mod r#macro;
pub mod mm;
pub mod realm;
pub mod rmi;
pub mod rmm;
pub mod rsi;
pub mod smc;

pub(crate) mod utils; // TODO: move to lib

#[macro_use]
extern crate log;

use crate::rmi::RMI;
use crate::rmm::PageMap;
use crate::smc::SecureMonitorCall;

pub struct Monitor {
    pub rmi: RMI,
    pub smc: SecureMonitorCall,
    pub mm: PageMap,
}

impl Monitor {
    pub fn new(rmi: RMI, smc: SecureMonitorCall, mm: PageMap) -> Self {
        Self { rmi, smc, mm }
    }
}
