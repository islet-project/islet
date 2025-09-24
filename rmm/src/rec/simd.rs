use aarch64_cpu::registers::{Readable, Writeable};
use armv9a::regs::{CPTR_EL2, SMCR_EL2, SVCR, ZCR_EL1, ZCR_EL2};
use armv9a::InMemoryRegister;
use core::arch::asm;
use core::array::from_fn;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use super::Rec;
use crate::config::NUM_OF_CPU;
use crate::cpu::get_cpu_id;
use crate::realm::rd::Rd;
use crate::rmi::error::Error;
use crate::simd::{sme_en, SimdConfig, ZCR_EL2_LEN_WIDTH};

// SIMD context structure
#[derive(Default, Debug)]
pub struct SimdRegister {
    pub is_used: bool,
    pub is_saved: bool,

    pub cfg: SimdConfig,
    // EL2 trap register for this context
    pub cptr_el2: u64,
    // SME specific Streaming vector control register
    pub svcr: u64,
    // SIMD data registers
    pub fpu: FpuRegs,
    pub sve: SveRegs,
}

// FPU registers
const NUM_FPU_REGS: usize = 32;
#[derive(Default, Debug)]
pub struct FpuRegs {
    pub q: [u128; NUM_FPU_REGS],
    // SIMD related control and status sysregs
    pub fpsr: u64,
    pub fpcr: u64,
}

lazy_static! {
    static ref NS_SIMD: [Mutex<SimdRegister>; NUM_OF_CPU] =
        from_fn(|_| Mutex::new(SimdRegister::default()));
}

// SVE registers
const NUM_VECTOR_REGS: usize = 32;
const NUM_PREDICATE_REGS: usize = 16;
#[derive(Default, Debug)]
pub struct SveRegs {
    // lower 128bits of each z register are shared with v
    // implementation-defined lengh: 128bits~2048bits. get it from zcr_el2
    pub z: [[u128; NUM_VECTOR_REGS]; ZCR_EL2_LEN_WIDTH as usize],
    // Each predicate register is 1/8 of the Zx length.
    pub p: [[u16; NUM_PREDICATE_REGS]; ZCR_EL2_LEN_WIDTH as usize],
    pub ffr: u64,
    pub zcr_el2: u64,
    pub zcr_el12: u64,
}

// TODO: Save according to the hint in FID with SMCCCv1.3 or v1.4

// SIMD context initialization function
pub fn init_simd(rec: &mut Rec<'_>) -> Result<(), Error> {
    let raw_ptr: *const Rd = rec.owner()? as *const Rd;
    let rd: &Rd = unsafe { raw_ptr.as_ref().expect("REASON") }; // FIXME
    let simd_cfg = rd.simd_config();

    let mut zcr_el2: u64 = 0;
    let mut svcr: u64 = 0;

    rec.context.simd.is_used = false;
    rec.context.simd.is_saved = false;
    rec.context.simd.cfg.sve_en = simd_cfg.sve_en;
    rec.context.simd.cfg.sve_vq = simd_cfg.sve_vq;
    rec.context.simd.cfg.sme_en = simd_cfg.sme_en;

    // Initialize SVE related fields and config registers
    if simd_cfg.sve_en {
        zcr_el2 = ZCR_EL2::LEN.val(simd_cfg.sve_vq).value;
    }
    if simd_cfg.sme_en {
        svcr = 0;
    }

    let simd_regs = &mut rec.context.simd;
    // Note: As islet-rmm doesn't enable VHE in the realm world as following,
    // HCR_EL2.E2H=0, HCR_EL2.TGE=0
    // the layout of CPTR_EL2 for non E2H is used.
    simd_regs.cptr_el2 =
        (CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET + CPTR_EL2::TFP::SET + CPTR_EL2::TZ::SET).value;
    simd_regs.sve.zcr_el2 = zcr_el2;
    simd_regs.svcr = svcr;
    Ok(())
}

// Note: Put assembly instructions only in the functions below.
//   Do not add log messages in functions with simd features.
//   RMM doesn't maintain its own SIMD context. However, compiler
//   tries to utilize simd registers in a function with the 'neon'
//   or 'sve' feature and it may results state corruption.

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[target_feature(enable = "neon")]
unsafe fn save_fpu(fpu: &mut FpuRegs) {
    let addr_q: u64 = fpu.q.as_ptr() as u64;
    unsafe {
        asm!(
        "stp q0, q1, [{addr_q}]",
        "stp q2, q3, [{addr_q}, #32]",
        "stp q4, q5, [{addr_q}, #64]",
        "stp q6, q7, [{addr_q}, #96]",
        "stp q8, q9, [{addr_q}, #128]",
        "stp q10, q11, [{addr_q}, #160]",
        "stp q12, q13, [{addr_q}, #192]",
        "stp q14, q15, [{addr_q}, #224]",
        "stp q16, q17, [{addr_q}, #256]",
        "stp q18, q19, [{addr_q}, #288]",
        "stp q20, q21, [{addr_q}, #320]",
        "stp q22, q23, [{addr_q}, #352]",
        "stp q24, q25, [{addr_q}, #384]",
        "stp q26, q27, [{addr_q}, #416]",
        "stp q28, q29, [{addr_q}, #448]",
        "stp q30, q31, [{addr_q}, #480]",
        addr_q = in(reg) addr_q,
        );
        save_fpu_crsr(fpu);
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[target_feature(enable = "neon")]
unsafe fn save_fpu_crsr(fpu: &mut FpuRegs) {
    let fpsr: u64;
    let fpcr: u64;
    unsafe {
        asm!(
        "mrs {fpsr}, fpsr",
        "mrs {fpcr}, fpcr",
        fpsr = out(reg) fpsr,
        fpcr = out(reg) fpcr,
        );
    }
    fpu.fpsr = fpsr;
    fpu.fpcr = fpcr;
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[target_feature(enable = "neon")]
pub unsafe fn restore_fpu(fpu: &FpuRegs) {
    let addr_q: u64 = fpu.q.as_ptr() as u64;
    unsafe {
        asm!(
             "ldp q0, q1, [x0]",
             "ldp q2, q3, [{addr_q}, #32]",
             "ldp q4, q5, [{addr_q}, #64]",
             "ldp q6, q7, [{addr_q}, #96]",
             "ldp q8, q9, [{addr_q}, #128]",
             "ldp q10, q11, [{addr_q}, #160]",
             "ldp q12, q13, [{addr_q}, #192]",
             "ldp q14, q15, [{addr_q}, #224]",
             "ldp q16, q17, [{addr_q}, #256]",
             "ldp q18, q19, [{addr_q}, #288]",
             "ldp q20, q21, [{addr_q}, #320]",
             "ldp q22, q23, [{addr_q}, #352]",
             "ldp q24, q25, [{addr_q}, #384]",
             "ldp q26, q27, [{addr_q}, #416]",
             "ldp q28, q29, [{addr_q}, #448]",
             "ldp q30, q31, [{addr_q}, #480]",
             addr_q = in(reg) addr_q,
        );
        restore_fpu_crsr(fpu);
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[target_feature(enable = "neon")]
pub unsafe fn restore_fpu_crsr(fpu: &FpuRegs) {
    unsafe {
        asm!(
             "msr fpsr, {fpsr}",
             "msr fpcr, {fpcr}",
             fpsr = in(reg) fpu.fpsr,
             fpcr = in(reg) fpu.fpcr,
        );
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[target_feature(enable = "sve")]
unsafe fn save_sve(sve: &mut SveRegs, save_ffr: bool) {
    let addr_z: u64 = sve.z.as_ptr() as u64;
    let addr_p: u64 = sve.p.as_ptr() as u64;
    unsafe {
        // Save the z regster bank
        asm!(
            "str z0, [{addr_z}, #0, MUL VL]",
            "str z1, [{addr_z}, #1, MUL VL]",
            "str z2, [{addr_z}, #2, MUL VL]",
            "str z3, [{addr_z}, #3, MUL VL]",
            "str z4, [{addr_z}, #4, MUL VL]",
            "str z5, [{addr_z}, #5, MUL VL]",
            "str z6, [{addr_z}, #6, MUL VL]",
            "str z7, [{addr_z}, #7, MUL VL]",
            "str z8, [{addr_z}, #8, MUL VL]",
            "str z9, [{addr_z}, #9, MUL VL]",
            "str z10, [{addr_z}, #10, MUL VL]",
            "str z11, [{addr_z}, #11, MUL VL]",
            "str z12, [{addr_z}, #12, MUL VL]",
            "str z13, [{addr_z}, #13, MUL VL]",
            "str z14, [{addr_z}, #14, MUL VL]",
            "str z15, [{addr_z}, #15, MUL VL]",
            "str z16, [{addr_z}, #16, MUL VL]",
            "str z17, [{addr_z}, #17, MUL VL]",
            "str z18, [{addr_z}, #18, MUL VL]",
            "str z19, [{addr_z}, #19, MUL VL]",
            "str z20, [{addr_z}, #20, MUL VL]",
            "str z21, [{addr_z}, #21, MUL VL]",
            "str z22, [{addr_z}, #22, MUL VL]",
            "str z23, [{addr_z}, #23, MUL VL]",
            "str z24, [{addr_z}, #24, MUL VL]",
            "str z25, [{addr_z}, #25, MUL VL]",
            "str z26, [{addr_z}, #26, MUL VL]",
            "str z27, [{addr_z}, #27, MUL VL]",
            "str z28, [{addr_z}, #28, MUL VL]",
            "str z29, [{addr_z}, #29, MUL VL]",
            "str z30, [{addr_z}, #30, MUL VL]",
            "str z31, [{addr_z}, #31, MUL VL]",
            addr_z = in(reg) addr_z,
        );
        // Save the p register bank
        asm!(
            "str p0, [{addr_p}, #0, MUL VL]",
            "str p1, [{addr_p}, #1, MUL VL]",
            "str p2, [{addr_p}, #2, MUL VL]",
            "str p3, [{addr_p}, #3, MUL VL]",
            "str p4, [{addr_p}, #4, MUL VL]",
            "str p5, [{addr_p}, #5, MUL VL]",
            "str p6, [{addr_p}, #6, MUL VL]",
            "str p7, [{addr_p}, #7, MUL VL]",
            "str p8, [{addr_p}, #8, MUL VL]",
            "str p9, [{addr_p}, #9, MUL VL]",
            "str p10, [{addr_p}, #10, MUL VL]",
            "str p11, [{addr_p}, #11, MUL VL]",
            "str p12, [{addr_p}, #12, MUL VL]",
            "str p13, [{addr_p}, #13, MUL VL]",
            "str p14, [{addr_p}, #14, MUL VL]",
            "str p15, [{addr_p}, #15, MUL VL]",
            addr_p = in(reg) addr_p,
        );
        if save_ffr {
            let addr_ffr: u64 = core::ptr::addr_of!(sve.ffr) as u64;
            asm!(
                "rdffr p0.B",
                "str p0, [{addr_ffr}]",
                addr_ffr = in(reg) addr_ffr,
            );
        }
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[inline(never)]
#[target_feature(enable = "sve")]
pub unsafe fn restore_sve(sve: &SveRegs, restore_ffr: bool) {
    let addr_z: u64 = sve.z.as_ptr() as u64;
    let addr_p: u64 = sve.p.as_ptr() as u64;
    unsafe {
        // Restore the z regster bank
        asm!(
            "ldr z0, [{addr_z}, #0, MUL VL]",
            "ldr z1, [{addr_z}, #1, MUL VL]",
            "ldr z2, [{addr_z}, #2, MUL VL]",
            "ldr z3, [{addr_z}, #3, MUL VL]",
            "ldr z4, [{addr_z}, #4, MUL VL]",
            "ldr z5, [{addr_z}, #5, MUL VL]",
            "ldr z6, [{addr_z}, #6, MUL VL]",
            "ldr z7, [{addr_z}, #7, MUL VL]",
            "ldr z8, [{addr_z}, #8, MUL VL]",
            "ldr z9, [{addr_z}, #9, MUL VL]",
            "ldr z10, [{addr_z}, #10, MUL VL]",
            "ldr z11, [{addr_z}, #11, MUL VL]",
            "ldr z12, [{addr_z}, #12, MUL VL]",
            "ldr z13, [{addr_z}, #13, MUL VL]",
            "ldr z14, [{addr_z}, #14, MUL VL]",
            "ldr z15, [{addr_z}, #15, MUL VL]",
            "ldr z16, [{addr_z}, #16, MUL VL]",
            "ldr z17, [{addr_z}, #17, MUL VL]",
            "ldr z18, [{addr_z}, #18, MUL VL]",
            "ldr z19, [{addr_z}, #19, MUL VL]",
            "ldr z20, [{addr_z}, #20, MUL VL]",
            "ldr z21, [{addr_z}, #21, MUL VL]",
            "ldr z22, [{addr_z}, #22, MUL VL]",
            "ldr z23, [{addr_z}, #23, MUL VL]",
            "ldr z24, [{addr_z}, #24, MUL VL]",
            "ldr z25, [{addr_z}, #25, MUL VL]",
            "ldr z26, [{addr_z}, #26, MUL VL]",
            "ldr z27, [{addr_z}, #27, MUL VL]",
            "ldr z28, [{addr_z}, #28, MUL VL]",
            "ldr z29, [{addr_z}, #29, MUL VL]",
            "ldr z30, [{addr_z}, #30, MUL VL]",
            "ldr z31, [{addr_z}, #31, MUL VL]",
            addr_z = in(reg) addr_z,
        );

        if restore_ffr {
            let addr_ffr: u64 = core::ptr::addr_of!(sve.ffr) as u64;
            asm!(
                "ldr p0, [{addr_ffr}]",
                "wrffr p0.B",
                addr_ffr = in(reg) addr_ffr,
            );
        }
        // Restore the p register bank
        asm!(
        "ldr p0, [{addr_p}, #0, MUL VL]",
        "ldr p1, [{addr_p}, #1, MUL VL]",
        "ldr p2, [{addr_p}, #2, MUL VL]",
        "ldr p3, [{addr_p}, #3, MUL VL]",
        "ldr p4, [{addr_p}, #4, MUL VL]",
        "ldr p5, [{addr_p}, #5, MUL VL]",
        "ldr p6, [{addr_p}, #6, MUL VL]",
        "ldr p7, [{addr_p}, #7, MUL VL]",
        "ldr p8, [{addr_p}, #8, MUL VL]",
        "ldr p9, [{addr_p}, #9, MUL VL]",
        "ldr p10, [{addr_p}, #10, MUL VL]",
        "ldr p11, [{addr_p}, #11, MUL VL]",
        "ldr p12, [{addr_p}, #12, MUL VL]",
        "ldr p13, [{addr_p}, #13, MUL VL]",
        "ldr p14, [{addr_p}, #14, MUL VL]",
        "ldr p15, [{addr_p}, #15, MUL VL]",
        addr_p = in(reg) addr_p,
        );
    }
}

fn preserve_ffr(svcr: u64) -> bool {
    let svcr: InMemoryRegister<u64, SVCR::Register> = InMemoryRegister::new(svcr);
    let mut rtn = true;

    let is_streaming = sme_en() && svcr.read(SVCR::SM) != 0;
    if is_streaming {
        rtn = SMCR_EL2.read(SMCR_EL2::FA64) != 0;
    }
    rtn
}

// This function is called when a SIMD access in Realm
// is made for the first time since REC_ENTER.
// See exception/trap.rs.
pub fn restore_state_lazy(rec: &Rec<'_>) {
    let rec_simd = &rec.context.simd;
    let mut ns_simd = NS_SIMD[get_cpu_id()].lock();

    // Disable simd traps during the context mgmt.
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    if rec_simd.cfg.sve_en {
        ns_simd.sve.zcr_el2 = ZCR_EL2.get();
        ns_simd.sve.zcr_el12 = ZCR_EL1.get();
        let mut max_len = ns_simd.sve.zcr_el2;
        if max_len < rec_simd.sve.zcr_el2 {
            max_len = rec_simd.sve.zcr_el2;
        }
        // Save at max to prevent leakage across worlds
        ZCR_EL2.set(max_len);
        #[cfg(not(any(test, miri, fuzzing)))]
        unsafe {
            // ns_simd.svcr is save at restore_state() on REC_ENTER
            let save_ffr = preserve_ffr(ns_simd.svcr);
            save_sve(&mut ns_simd.sve, save_ffr);
            save_fpu_crsr(&mut ns_simd.fpu);
            if sme_en() {
                SVCR.set(rec_simd.svcr);
            }
            if rec_simd.is_saved {
                let restore_ffr = true; // Sinde Realm is not supported with SME, it's always true.
                restore_sve(&rec_simd.sve, restore_ffr);
                restore_fpu_crsr(&rec_simd.fpu);
            }
        }
        ZCR_EL2.set(rec_simd.sve.zcr_el2);
        ZCR_EL1.set(rec_simd.sve.zcr_el12);
    } else {
        unsafe {
            save_fpu(&mut ns_simd.fpu);
            if rec_simd.is_saved {
                restore_fpu(&rec_simd.fpu);
            }
        }
    }
}

pub fn restore_state(rec: &Rec<'_>) {
    let rec_simd = &rec.context.simd;
    let mut ns_simd = NS_SIMD[get_cpu_id()].lock();

    // Disable simd traps during the context mgmt.
    CPTR_EL2.write(CPTR_EL2::TAM::SET);

    // We don't save/restore any state
    // until SIMD registers are actually accessed in Realms.

    if sme_en() {
        ns_simd.svcr = SVCR.get();
        // Note: Don't call SVCR.set(rec_simd.svcr) here.
        // Otherwise, we will loose unsaved NS context.
        // SVCR's SM bit change between 0 and 1, one way or the other,
        // results in setting the whole SIMD registers to zeros.
    }
    ns_simd.cptr_el2 = CPTR_EL2.get();
    CPTR_EL2.set(rec_simd.cptr_el2);
}

pub fn save_state(rec: &mut Rec<'_>) {
    let rec_simd = &mut rec.context.simd;
    let ns_simd = NS_SIMD[get_cpu_id()].lock();

    rec_simd.cptr_el2 =
        (CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET + CPTR_EL2::TFP::SET + CPTR_EL2::TZ::SET).value;
    if !rec_simd.is_used {
        CPTR_EL2.set(ns_simd.cptr_el2);
        if sme_en() {
            SVCR.set(ns_simd.svcr);
        }
        return;
    }
    // Disable simd traps during the context mgmt.
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    // Since FEAT_SME is not for Realms, no need to store SVCR which doesn't change.

    if rec_simd.cfg.sve_en {
        rec_simd.sve.zcr_el2 = ZCR_EL2.get();
        rec_simd.sve.zcr_el12 = ZCR_EL1.get();
        let mut max_len = ns_simd.sve.zcr_el2;
        if max_len < rec_simd.sve.zcr_el2 {
            max_len = rec_simd.sve.zcr_el2;
        }
        // Save context at maximum to prevent leakage
        ZCR_EL2.set(max_len);
        unsafe {
            let save_ffr = true; // Since FEAT_SME is not for Realms, it's always true.
            save_sve(&mut rec_simd.sve, save_ffr);
            save_fpu_crsr(&mut rec_simd.fpu);
            // Set SVCR before loading context.
            // Otherwise, when SVCR:SM is 0, all simd registers are set to zero.
            if sme_en() {
                SVCR.set(ns_simd.svcr);
            }
            let restore_ffr = preserve_ffr(ns_simd.svcr);
            restore_sve(&ns_simd.sve, restore_ffr);
            restore_fpu_crsr(&ns_simd.fpu);
        }
        ZCR_EL2.set(ns_simd.sve.zcr_el2);
        ZCR_EL1.set(ns_simd.sve.zcr_el12);
    } else {
        // For SIMD and FPU
        unsafe {
            save_fpu(&mut rec_simd.fpu);
            restore_fpu(&ns_simd.fpu);
        }
    }
    // To maintain immutability of rec.context during its restoration,
    // update the context here in advance.
    rec_simd.is_used = false;
    rec_simd.is_saved = true;
    CPTR_EL2.set(ns_simd.cptr_el2);
}
