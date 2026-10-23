use aarch64_cpu::registers::{Readable, Writeable};
use armv9a::regs::{CPTR_EL2, SMCR_EL2, SVCR, ZCR_EL1, ZCR_EL2};
use armv9a::InMemoryRegister;
use core::arch::asm;
use core::array::from_fn;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use super::Rec;
use crate::asm::isb;
use crate::config::NUM_OF_CPU;
use crate::cpu::get_cpu_id;
use crate::granule::GranuleState;
use crate::realm::rd::Rd;
use crate::rec::RecAuxIndex;
use crate::rmi::error::Error;
use crate::simd::{sme_en, sve_en as host_sve_en, SimdConfig, MAX_VQ};
use crate::{get_granule, get_granule_if};

// SIMD context structure
#[derive(Default, Debug)]
pub struct SimdContext {
    pub is_used: bool,
    pub is_saved: bool,

    pub cfg: SimdConfig,
}

#[derive(Default, Debug)]
pub struct SimdRegister {
    // SME specific Streaming vector control register
    pub svcr: u64,
    // SIMD data registers
    pub fpu: FpuRegs,
    pub sve: SveRegs,
}

impl SimdRegister {
    pub fn new() -> Self {
        Default::default()
    }
}

impl vmsa::guard::Content for SimdRegister {}
impl safe_abstraction::raw_ptr::RawPtr for SimdRegister {}
impl safe_abstraction::raw_ptr::SafetyChecked for SimdRegister {}
impl safe_abstraction::raw_ptr::SafetyAssured for SimdRegister {
    fn is_initialized(&self) -> bool {
        // It is wiped out with zero's on granule delegation.
        // Then, SimdRegister gets initialized on Rec::init() call.
        true
    }

    fn verify_ownership(&self) -> bool {
        true
    }
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
    pub z: [[u128; NUM_VECTOR_REGS]; MAX_VQ as usize],
    // Each predicate register is 1/8 of the Zx length.
    pub p: [[u16; NUM_PREDICATE_REGS]; MAX_VQ as usize],
    pub ffr: [u16; MAX_VQ as usize],
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

    let simd_aux = rec.aux(RecAuxIndex::SIMD as usize) as usize;
    let mut simd_granule = get_granule_if!(simd_aux, GranuleState::RecAux)?;
    debug!("RecAux granule for simd at 0x{:x}", simd_aux);
    let mut simd_regs = simd_granule.new_uninit_with::<SimdRegister>(SimdRegister::new())?;
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
/// Use neon only for (re)storing Rec's simd context. The q registers are
/// saved in pairs via `stp`, i.e. at fixed byte offsets of 32 bytes
/// (2 * a 16-byte register); those offsets are asm literals by design.
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
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
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
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
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "neon")]
unsafe fn restore_fpu(fpu: &FpuRegs) {
    let addr_q: u64 = fpu.q.as_ptr() as u64;
    unsafe {
        asm!(
            "ldp q0, q1, [{addr_q}]",
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
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "neon")]
unsafe fn restore_fpu_crsr(fpu: &FpuRegs) {
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
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn save_sve(sve: &mut SveRegs, save_ffr: bool) {
    let addr_z: u64 = sve.z.as_ptr() as u64;
    let addr_p: u64 = sve.p.as_ptr() as u64;
    unsafe {
        // Save the z register bank
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
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn restore_sve(sve: &SveRegs, restore_ffr: bool) {
    let addr_z: u64 = sve.z.as_ptr() as u64;
    let addr_p: u64 = sve.p.as_ptr() as u64;
    unsafe {
        // Restore the z register bank
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

// Returns true if the PE implements FEAT_SME and the given SVCR value indicates
// Streaming SVE mode (SVCR.SM == 1).
//
// In Armv9-A, FEAT_SME is architecturally independent of non-streaming FEAT_SVE:
// a PE may implement FEAT_SME without FEAT_SVE (`host_sve_en() == false` and
// `sme_en() == true`). When `SVCR.SM == 1` on any SME-capable PE, the full
// Streaming SVE register file (`Z0..Z31` of width `SVL`, `P0..P15`, and `FFR`
// when `FEAT_SME_FA64` is enabled) is active and must be saved/restored using
// SVE instructions rather than 128-bit FPU `Q0..Q31` loads/stores.
fn is_streaming(svcr: u64) -> bool {
    let svcr: InMemoryRegister<u64, SVCR::Register> = InMemoryRegister::new(svcr);
    sme_en() && svcr.read(SVCR::SM) != 0
}

// In Streaming SVE mode (`SVCR.SM == 1`), `rdffr`/`wrffr` are legal only if the
// full A64 instruction set is enabled in streaming mode (`SMCR_EL2.FA64 == 1`).
// In normal SVE mode (`SVCR.SM == 0`), `FFR` is always accessible.
fn preserve_ffr(svcr: u64) -> bool {
    if is_streaming(svcr) {
        SMCR_EL2.read(SMCR_EL2::FA64) != 0
    } else {
        true
    }
}

// This function is called when a SIMD access in Realm
// is made for the first time since REC_ENTER.
// See exception/trap.rs.
pub fn restore_state_lazy(rec: &Rec<'_>) {
    let rec_simd_ctx = &rec.context.simd;
    let simd_aux = rec.aux(RecAuxIndex::SIMD as usize) as usize;
    let simd_granule = match get_granule_if!(simd_aux, GranuleState::RecAux) {
        Ok(guard) => guard,
        Err(_e) => {
            error!("Unable to get RecAux granule at 0x{:x}", simd_aux);
            return;
        }
    };
    let rec_simd = simd_granule.content::<SimdRegister>().unwrap();
    let mut ns_simd = NS_SIMD[get_cpu_id()].lock();

    // Disable simd traps during the context mgmt and synchronize before
    // accessing any SIMD/SVE/SME registers at EL2.
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    isb();

    // Capture the NS world's SVCR before saving NS registers (which checks
    // SVCR.SM via is_streaming() / preserve_ffr()) and before switching SVCR
    // to the REC's value.
    if sme_en() {
        ns_simd.svcr = SVCR.get();
    }

    // 1. Save the Non-Secure world's SIMD/SVE state.
    // The full SVE register set (Z0..Z31, P0..P15, FFR) must be saved instead
    // of just 128-bit Q0..Q31 in two cases:
    //   a) The PE supports non-streaming SVE (`host_sve_en()`): loading Q0..Q31
    //      for a non-SVE Realm zeroes the upper bits [VL_max-1:128] of Z0..Z31,
    //      so the full NS SVE width must be preserved even if this Realm has
    //      SVE disabled.
    //   b) The PE is an SME-only CPU (`host_sve_en() == false`, `sme_en() == true`)
    //      and the NS world entered RMM in Streaming SVE mode (`is_streaming(ns_simd.svcr)`):
    //      NS is actively using the streaming Z0..Z31, P0..P15, and FFR registers,
    //      which will be wiped by hardware as soon as SVCR.SM is cleared to 0
    //      for the Realm in step 2 below.
    if host_sve_en() || is_streaming(ns_simd.svcr) {
        // Guard ZCR_EL1 / ZCR_EL2 accesses with `host_sve_en()`: on an SME-only PE
        // (`host_sve_en() == false`), ZCR_EL1 and ZCR_EL2 are unimplemented and
        // accessing them raises an UNDEFINED exception at EL2. In Streaming SVE
        // mode, the active vector length (SVL) is controlled by SMCR_EL2.LEN
        // (which EL3 leaves untouched from NS-EL2), not by ZCR_EL2.LEN.
        if host_sve_en() {
            ns_simd.sve.zcr_el12 = ZCR_EL1.get();
            ns_simd.sve.zcr_el2 = ZCR_EL2.get();
            ZCR_EL2.write(ZCR_EL2::LEN.val(MAX_VQ - 1));
            isb();
        }
        unsafe {
            let save_ffr = preserve_ffr(ns_simd.svcr);
            save_sve(&mut ns_simd.sve, save_ffr);
            save_fpu_crsr(&mut ns_simd.fpu);
        }
    } else {
        unsafe {
            save_fpu(&mut ns_simd.fpu);
            save_fpu_crsr(&mut ns_simd.fpu);
        }
    }

    // 2. Switch SVCR to the REC's value (0) after saving NS registers and
    // before restoring REC registers, since toggling SVCR.SM zeroes the
    // vector registers in hardware.
    if sme_en() {
        SVCR.set(rec_simd.svcr);
        isb();
    }

    // 3. Restore the REC's SIMD/SVE state unconditionally (even when
    // !is_saved, as the zeroed AUX granules hold the REC's architectural
    // initial state, preventing NS register leakage into the Realm).
    if rec_simd_ctx.cfg.sve_en {
        // Install the REC's VL cap in ZCR_EL2 and the REC's ZCR_EL1 now
        // (after NS SVE state has been saved) and synchronize before
        // restore_sve() derives its MUL VL strides.
        ZCR_EL2.set(rec_simd.sve.zcr_el2);
        ZCR_EL1.set(rec_simd.sve.zcr_el12);
        isb();
        unsafe {
            let restore_ffr = true; // Since Realm is not supported with SME, it's always true.
            restore_sve(&rec_simd.sve, restore_ffr);
            restore_fpu_crsr(&rec_simd.fpu);
        }
        // Leave TFP=0 and TZ=0 for the Realm, but re-enable TSM=1 since SME
        // is not supported for Realms.
        CPTR_EL2.write(CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET);
    } else {
        unsafe {
            restore_fpu(&rec_simd.fpu);
            restore_fpu_crsr(&rec_simd.fpu);
        }
        // Leave TFP=0 for the Realm, but keep TZ=1 and TSM=1 so SVE and SME
        // instructions remain trapped for this non-SVE Realm.
        CPTR_EL2.write(CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET + CPTR_EL2::TZ::SET);
    }
    isb();
}

pub fn restore_state(_rec: &Rec<'_>) {
    // We don't save/restore any SIMD/SVE registers, ZCR_EL1/ZCR_EL2, or SVCR
    // until SIMD/SVE is actually accessed in the Realm. Setting TFP=1, TZ=1,
    // and TSM=1 below traps all FP/SVE/SME instructions as well as direct
    // accesses to ZCR_EL1/ZCR_EL2/SVCR into restore_state_lazy().
    // Note: As islet-rmm doesn't enable VHE in the realm world (HCR_EL2.E2H=0,
    // HCR_EL2.TGE=0), the non-E2H layout of CPTR_EL2 is used.
    CPTR_EL2
        .write(CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET + CPTR_EL2::TFP::SET + CPTR_EL2::TZ::SET);
    isb();
}

pub fn save_state(rec: &mut Rec<'_>) {
    let simd_aux = rec.aux(RecAuxIndex::SIMD as usize) as usize;
    let rec_simd_ctx = &mut rec.context.simd;

    // Disable SIMD traps for RMM context management (restoring RMM's default
    // CPTR_EL2 = TAM::SET configuration from setup_el2()).
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    isb();

    if !rec_simd_ctx.is_used {
        return;
    }
    // Since FEAT_SME is not for Realms, no need to store SVCR which doesn't change.

    let mut simd_granule = match get_granule_if!(simd_aux, GranuleState::RecAux) {
        Ok(guard) => guard,
        Err(_e) => {
            error!("Unable to get RecAux granule at 0x{:x}", simd_aux);
            return;
        }
    };
    let mut rec_simd = simd_granule.content_mut::<SimdRegister>().unwrap();
    let ns_simd = NS_SIMD[get_cpu_id()].lock();

    // 1. Save the REC's SIMD/SVE state.
    if rec_simd_ctx.cfg.sve_en {
        // ZCR_EL2 still holds rec_simd.sve.zcr_el2 here, so save_sve() saves
        // at the REC's configured vector length.
        rec_simd.sve.zcr_el12 = ZCR_EL1.get();
        unsafe {
            let save_ffr = true; // Since FEAT_SME is not for Realms, it's always true.
            save_sve(&mut rec_simd.sve, save_ffr);
            save_fpu_crsr(&mut rec_simd.fpu);
        }
    } else {
        unsafe {
            save_fpu(&mut rec_simd.fpu);
            save_fpu_crsr(&mut rec_simd.fpu);
        }
    }

    // 2. Restore NS SVCR before loading NS SIMD/SVE registers.
    // Otherwise, toggling SVCR.SM between 0 and 1 zeroes all SIMD registers.
    if sme_en() {
        SVCR.set(ns_simd.svcr);
        isb();
    }

    // 3. Restore the Non-Secure world's SIMD/SVE state.
    // Matches the condition in `restore_state_lazy()`: restore from `ns_simd.sve`
    // whenever the PE supports non-streaming SVE (`host_sve_en()`) or when
    // returning to NS in Streaming SVE mode (`is_streaming(ns_simd.svcr)`,
    // including on SME-only PEs where `host_sve_en() == false`).
    // Note that `SVCR` has already been restored in step 2 above, so if
    // `is_streaming(ns_simd.svcr)` is true, the PE is now in Streaming SVE
    // mode and `restore_sve()` will reload the streaming Z/P/FFR registers.
    if host_sve_en() || is_streaming(ns_simd.svcr) {
        // Only touch ZCR_EL2 / ZCR_EL1 when non-streaming SVE is implemented
        // on the PE; on an SME-only PE (`host_sve_en() == false`), ZCR_ELx
        // registers do not exist and would trap as UNDEFINED.
        if host_sve_en() {
            ZCR_EL2.write(ZCR_EL2::LEN.val(MAX_VQ - 1));
            isb();
        }
        unsafe {
            let restore_ffr = preserve_ffr(ns_simd.svcr);
            restore_sve(&ns_simd.sve, restore_ffr);
            restore_fpu_crsr(&ns_simd.fpu);
        }
        if host_sve_en() {
            ZCR_EL2.set(ns_simd.sve.zcr_el2);
            ZCR_EL1.set(ns_simd.sve.zcr_el12);
            isb();
        }
    } else {
        unsafe {
            restore_fpu(&ns_simd.fpu);
            restore_fpu_crsr(&ns_simd.fpu);
        }
    }

    // To maintain immutability of rec.context during its restoration,
    // update the context here in advance.
    rec_simd_ctx.is_used = false;
    rec_simd_ctx.is_saved = true;
}
