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
#[macro_use]
pub mod gic;
pub mod granule;
#[macro_use]
pub mod host;
pub mod io;
pub mod logger;
pub mod mm;
#[cfg(not(test))]
pub mod panic;
pub mod realm;
pub mod rmi;
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

use armv9a::{bits_in_reg, regs::*};

pub struct Monitor {
    pub rmi: RMI,
    pub rsi: RsiHandle,
}

impl Monitor {
    pub fn new(rmi: RMI) -> Self {
        Self {
            rmi,
            rsi: RsiHandle::new(),
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

/// Call `rmm_exit` within `exception/vectors.s` and jumps to EL1.
///
/// Currently, this function gets [0usize; 3] as an argument to initialize
/// x0, x1 and x2 registers.
///
/// When an exception occurs and the flow comes back to EL2 through `rmm_enter`,
/// x0, x1 and x2 registers might be changed to contain additional information
/// set from `handle_lower_exception`.
/// These are the return values of this function.
/// The return value encodes: [rmi::RET_XXX, ret_val1, ret_val2]
/// In most cases, the function returns [rmi::RET_SUCCESS, _, _]
/// pagefault returns [rmi::RET_PAGE_FAULT, faulted address, _]
pub unsafe fn rmm_exit(args: [usize; 4]) -> [usize; 4] {
    let mut ret: [usize; 4] = [0usize; 4];

    core::arch::asm!(
        "bl rmm_exit",
        inlateout("x0") args[0] => ret[0],
        inlateout("x1") args[1] => ret[1],
        inlateout("x2") args[2] => ret[2],
        inlateout("x3") args[3] => ret[3],
    );

    ret
}
