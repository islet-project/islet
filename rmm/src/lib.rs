#![no_std]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

#[cfg(not(test))]
pub mod allocator;
pub mod asm;
pub mod config;
pub mod cpu;
pub mod error;
pub mod event;
pub mod exception;
pub mod gic;
pub mod helper;
#[macro_use]
pub mod host;
pub mod io;
pub mod logger;
pub mod mm;
#[cfg(not(test))]
pub mod panic;
pub mod realm;
pub mod rmi;
pub mod rmm;
pub mod rsi;
#[macro_use]
pub mod r#macro;

extern crate alloc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

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
