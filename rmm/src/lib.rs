#![no_std]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]

#[cfg(not(any(test, fuzzing)))]
pub mod allocator;
pub mod asm;
pub mod config;
pub(crate) mod cose;
pub mod cpu;
pub(crate) mod event;
pub mod exception;
pub mod gic;
#[macro_use]
pub mod granule;
#[macro_use]
pub(crate) mod host;
pub mod logger;
pub mod mm;
#[cfg(not(any(test, kani, miri, fuzzing)))]
pub mod panic;
pub mod realm;
pub mod rec;
pub mod rmi;
pub mod rsi;
pub mod simd;
#[cfg(feature = "stat")]
pub mod stat;
#[cfg(any(test, miri))]
pub(crate) mod test_utils;
#[cfg(fuzzing)]
pub mod test_utils;
#[macro_use]
pub mod r#macro;
mod measurement;
#[cfg(kani)]
// we declare monitor as `pub` in model checking, so that
// its member can be accessed freely outside the rmm crate
pub mod monitor;
#[cfg(not(kani))]
mod monitor;
mod rmm_el3;

extern crate alloc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use crate::config::PlatformMemoryLayout;
use crate::exception::vectors;
#[cfg(feature = "gst_page_table")]
use crate::granule::create_granule_status_table as setup_gst;
use crate::mm::translation::{get_page_table, init_page_table};
use crate::monitor::Monitor;
use crate::rmm_el3::setup_el3_ifc;

use aarch64_cpu::registers::*;
use core::ptr::addr_of;

#[cfg(not(kani))]
// model checking harnesses do not use this function, instead
// they use their own entry points marked with #[kani::proof]
// where slightly adjusted `Monitor` is used
/// Starts the RMM on the specified CPU with the given memory layout.
///
/// # Safety
///
/// The caller must ensure that:
/// - The caller must ensure that `cpu_id` corresponds to a valid and initialized CPU.
/// - The `layout` must be a valid `PlatformMemoryLayout` appropriate for the platform.
/// - Calling this function may alter system-level configurations and should be done with caution.
pub unsafe fn start(cpu_id: usize, layout: PlatformMemoryLayout) {
    let el3_shared_buf = layout.el3_shared_buf;
    setup_mmu_cfg(layout);
    setup_el2();
    #[cfg(feature = "gst_page_table")]
    setup_gst();
    // TODO: call once or with every start?
    if cpu_id == 0 {
        setup_el3_ifc(el3_shared_buf);
    }

    Monitor::new().run();
}

/// # Safety
///
/// This function performs several operations that involve writing to system control registers
/// at the EL2.
///
/// The caller must ensure that:
/// - The function is called at EL2 with the required privileges.
/// - The `vectors` variable points to a valid exception vector table in memory.
/// - The system is in a state where modifying these control registers is safe and will not
///   interfere with other critical operations.
///
/// Failing to meet these requirements can result in system crashes, security vulnerabilities,
/// or other undefined behavior.
unsafe fn setup_el2() {
    HCR_EL2.write(
        HCR_EL2::FWB::SET
            + HCR_EL2::TEA::SET
            + HCR_EL2::TERR::SET
            + HCR_EL2::TLOR::SET
            + HCR_EL2::RW::SET
            + HCR_EL2::TSW::SET
            + HCR_EL2::TACR::SET
            + HCR_EL2::TIDCP::SET
            + HCR_EL2::TSC::SET
            + HCR_EL2::TID3::SET
            + HCR_EL2::BSU::InnerShareable
            + HCR_EL2::FB::SET
            + HCR_EL2::AMO::SET
            + HCR_EL2::IMO::SET
            + HCR_EL2::FMO::SET
            + HCR_EL2::VM::SET
            + HCR_EL2::API::SET
            + HCR_EL2::APK::SET,
        // HCR_EL2::TWI::SET,
    );
    VBAR_EL2.set(addr_of!(vectors) as u64);
    SCTLR_EL2
        .write(SCTLR_EL2::C::SET + SCTLR_EL2::I::SET + SCTLR_EL2::M::SET + SCTLR_EL2::EOS::SET);
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    ICC_SRE_EL2.write(
        ICC_SRE_EL2::ENABLE::SET
            + ICC_SRE_EL2::DIB::SET
            + ICC_SRE_EL2::DFB::SET
            + ICC_SRE_EL2::SRE::SET,
    );
}

/// # Safety
///
/// This function configures the Memory Management Unit (MMU) at the EL2.
///
/// The caller must ensure:
/// - The function is called at EL2 with the appropriate privileges.
/// - The translation table base address (`ttbl_base`) is valid and correctly initialized.
/// - Modifying these registers and executing these assembly instructions will not interfere
///   with other critical operations.
///
/// Failing to meet these requirements can result in system crashes, memory corruption, security
/// vulnerabilities, or other undefined behavior.
unsafe fn setup_mmu_cfg(layout: PlatformMemoryLayout) {
    core::arch::asm!("tlbi alle2is", "dsb ish", "isb",);

    // /* Set attributes in the right indices of the MAIR. */
    let mair_el2 = MAIR_EL2::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL2::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL2::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck
        + MAIR_EL2::Attr2_Device::nonGathering_nonReordering_EarlyWriteAck;

    /*
     * The size of the virtual address space is configured as 64 – T0SZ.
     * In this, 64 – 0x19 gives 39 bits of virtual address space.
     * This equates to 512GB (2^39), which means that the entire virtual address
     * space is covered by a single L1 table.
     * Therefore, our starting level of translation is level 1.
     */
    let tcr_el2 = TCR_EL2::T0SZ.val(0x19)
        + TCR_EL2::PS::Bits_40
        + TCR_EL2::TG0::KiB_4
        + TCR_EL2::SH0::Inner
        + TCR_EL2::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL2::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable;

    // set the ttbl base address, this is where the memory address translation
    // table walk starts
    init_page_table(layout);
    let ttbl_base = get_page_table();

    // Invalidate the local I-cache so that any instructions fetched
    // speculatively are discarded.
    MAIR_EL2.write(mair_el2);
    TCR_EL2.write(tcr_el2);
    TTBR0_EL2.set(ttbl_base);
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
///
/// # Safety
///
/// - This function alters the processor's execution level by jumping to EL1;
///   the caller must ensure that the system is in a correct state for this transition.
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
