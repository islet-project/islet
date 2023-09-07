#![no_std]
#![allow(incomplete_features)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

pub mod asm;
pub mod error;
pub mod event;
#[macro_use]
pub mod host;
pub mod io;
pub mod logger;
#[macro_use]
pub mod r#macro;
pub mod mm;
pub mod realm;
pub mod rmi;
pub mod rmm;
pub mod rsi;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use crate::event::RsiHandle;
use crate::rmi::RMI;
use crate::rmm::PageMap;

pub struct Monitor {
    pub rmi: RMI,
    pub rsi: RsiHandle,
    pub mm: PageMap,
}

impl Monitor {
    pub fn new(rmi: RMI, mm: PageMap) -> Self {
        Self {
            rmi,
            rsi: RsiHandle::new(),
            mm,
        }
    }
}
