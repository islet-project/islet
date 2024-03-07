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
pub mod granule;
#[macro_use]
pub mod host;
pub mod io;
pub mod logger;
pub mod mm;
pub mod mmio;
#[cfg(not(any(test, kani)))]
pub mod panic;
pub mod realm;
pub mod rmi;
pub mod rsi;
pub mod rtt;
#[cfg(feature = "stat")]
pub mod stat;
#[macro_use]
pub mod r#macro;
mod measurement;
mod monitor;
mod rmm_el3;

extern crate alloc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use crate::exception::vectors;
#[cfg(feature = "gst_page_table")]
use crate::granule::create_granule_status_table as setup_gst;
use crate::mm::translation::get_page_table;
use crate::monitor::Monitor;
use crate::rmm_el3::setup_el3_ifc;

use armv9a::{bits_in_reg, regs::*};

pub unsafe fn start(cpu_id: usize) {
    setup_mmu_cfg();
    setup_el2();
    #[cfg(feature = "gst_page_table")]
    setup_gst();
    // TODO: call once or with every start?
    if cpu_id == 0 {
        setup_el3_ifc();
    }

    Monitor::new().run();
}

unsafe fn setup_el2() {
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
    SCTLR_EL2.set(SCTLR_EL2::C | SCTLR_EL2::I | SCTLR_EL2::M | SCTLR_EL2::EOS);
    CPTR_EL2.set(CPTR_EL2::TAM);
    ICC_SRE_EL2.set(ICC_SRE_EL2::ENABLE | ICC_SRE_EL2::DIB | ICC_SRE_EL2::DFB | ICC_SRE_EL2::SRE);
}

unsafe fn setup_mmu_cfg() {
    core::arch::asm!("tlbi alle2is", "dsb ish", "isb",);

    // /* Set attributes in the right indices of the MAIR. */
    let mair_el2 = bits_in_reg(MAIR_EL2::Attr0, mair_attr::NORMAL)
        | bits_in_reg(MAIR_EL2::Attr1, mair_attr::DEVICE_NGNRNE)
        | bits_in_reg(MAIR_EL2::Attr2, mair_attr::DEVICE_NGNRE);

    /*
     * The size of the virtual address space is configured as 64 – T0SZ.
     * In this, 64 – 0x19 gives 39 bits of virtual address space.
     * This equates to 512GB (2^39), which means that the entire virtual address
     * space is covered by a single L1 table.
     * Therefore, our starting level of translation is level 1.
     */
    let mut tcr_el2 = bits_in_reg(TCR_EL2::T0SZ, 0x19);

    // configure the tcr_el2 attributes
    tcr_el2 |= bits_in_reg(TCR_EL2::PS, tcr_paddr_size::PS_1T)
        | bits_in_reg(TCR_EL2::TG0, tcr_granule::G_4K)
        | bits_in_reg(TCR_EL2::SH0, tcr_shareable::INNER)
        | bits_in_reg(TCR_EL2::ORGN0, tcr_cacheable::WBWA)
        | bits_in_reg(TCR_EL2::IRGN0, tcr_cacheable::WBWA);

    // set the ttlb base address, this is where the memory address translation
    // table walk starts
    let ttlb_base = get_page_table();

    // Invalidate the local I-cache so that any instructions fetched
    // speculatively are discarded.
    MAIR_EL2.set(mair_el2);
    TCR_EL2.set(tcr_el2);
    TTBR0_EL2.set(ttlb_base);
    core::arch::asm!("dsb ish", "isb",);
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
#[cfg(not(kani))]
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

#[cfg(kani)]
pub unsafe fn rmm_exit(_args: [usize; 4]) -> [usize; 4] {
    let ret: [usize; 4] = [0usize; 4];
    ret
}
