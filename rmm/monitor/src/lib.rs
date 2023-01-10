#![no_std]
#![allow(incomplete_features)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

pub mod call;
pub mod communication;
pub mod error;
pub mod io;
pub mod logger;
pub mod r#macro;
pub mod mainloop;
pub mod mm;
pub mod realm;
pub mod rmi;

#[macro_use]
extern crate log;
