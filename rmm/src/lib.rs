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
use crate::exception::vectors;
use crate::rmi::RMI;
use crate::rmm::PageMap;

use armv9a::{bits_in_reg, regs::*};

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

pub unsafe fn init_el2() {
    HCR_EL2.set(
        HCR_EL2::FWB
            | HCR_EL2::TEA
            | HCR_EL2::TERR
            | HCR_EL2::TLOR
            | HCR_EL2::RW
            | HCR_EL2::TSW
            | HCR_EL2::TACR
            | HCR_EL2::TIDCP
            | HCR_EL2::TSC
            | HCR_EL2::TID3
            | (HCR_EL2::BSU & 0b01)
    //        | HCR_EL2::TWI
            | HCR_EL2::FB
            | HCR_EL2::AMO
            | HCR_EL2::IMO
            | HCR_EL2::FMO
            | HCR_EL2::VM,
    );
    VBAR_EL2.set(&vectors as *const u64 as u64);
    // FIXME: ACS got stuck when SCTLR_EL2 sets below flags at the same time.
    SCTLR_EL2.set(SCTLR_EL2::C);
    SCTLR_EL2.set(SCTLR_EL2::I | SCTLR_EL2::M | SCTLR_EL2::EOS);
    CPTR_EL2.set(CPTR_EL2::TAM);
    ICC_SRE_EL2.set(ICC_SRE_EL2::ENABLE | ICC_SRE_EL2::DIB | ICC_SRE_EL2::DFB | ICC_SRE_EL2::SRE);
    activate_stage2_mmu();
}

unsafe fn activate_stage2_mmu() {
    // stage 2 intitial table: L1 with 1024 entries (2 continuous 4KB pages)
    let vtcr_el2: u64 = bits_in_reg(VTCR_EL2::PS, tcr_paddr_size::PS_1T)
        | bits_in_reg(VTCR_EL2::TG0, tcr_granule::G_4K)
        | bits_in_reg(VTCR_EL2::SH0, tcr_shareable::INNER)
        | bits_in_reg(VTCR_EL2::ORGN0, tcr_cacheable::WBWA)
        | bits_in_reg(VTCR_EL2::IRGN0, tcr_cacheable::WBWA)
        | bits_in_reg(VTCR_EL2::SL0, tcr_start_level::L1)
        | bits_in_reg(VTCR_EL2::T0SZ, 24); // T0SZ, input address is 2^40 bytes

    // Invalidate the local I-cache so that any instructions fetched
    // speculatively are discarded.
    core::arch::asm!("ic iallu", "dsb nsh", "isb",);

    VTCR_EL2.set(vtcr_el2);

    core::arch::asm!("tlbi alle2", "dsb ish", "isb",);
}
