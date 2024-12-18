use aarch64_cpu::registers::ID_AA64PFR0_EL1;
use aarch64_cpu::registers::{Readable, Writeable};
use armv9a::regs::{CPTR_EL2, ID_AA64PFR1_SME_EL1, SMCR_EL2, ZCR_EL1, ZCR_EL2};
use core::arch::asm;
use core::array::from_fn;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use super::Rec;
use crate::config::NUM_OF_CPU;
use crate::cpu::get_cpu_id;
use crate::realm::rd::Rd;
use crate::rmi::error::Error;

// Vector length (VL) = size of a Z-register in bytes
// Vector quadwords (VQ) = size of a Z-register in units of 128 bits
// Minimum length of a SVE vector: 128 bits
const ZCR_EL2_LEN_WIDTH: u64 = 4;
const SVE_VQ_ARCH_MAX: u64 = (1 << ZCR_EL2_LEN_WIDTH) - 1;
const QUARD_WORD: u64 = 128;

#[derive(Default, Debug)]
// SIMD configuration structure
pub struct SimdConfig {
    // SVE enabled flag
    pub sve_en: bool,

    // SVE vector length represented in quads
    pub sve_vq: u64,

    // SME enabled flag
    pub sme_en: bool,
}

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

// SVE registers
const NUM_VECTOR_REGS: usize = 32;
const NUM_PREDICATE_REGS: usize = 16;
#[derive(Default, Debug)]
pub struct SveRegs {
    // lower 128bits of each z register are shared with v
    // implementation-defined lengh: 128bits~2048bits. zcr-el2에서 얻어와야...
    pub z: [u128; NUM_VECTOR_REGS],
    // Each predicate register is 1/8 of the Zx length.
    pub p: [u16; NUM_PREDICATE_REGS],
    pub ffr: u64,
    pub zcr_el2: u64,
    pub zcr_el12: u64,
}

lazy_static! {
    // TODO: storing ns simd context should be in the ns world, not in the realm world.
    static ref NS_SIMD: [Mutex<SimdRegister>; NUM_OF_CPU] = from_fn(|_| Mutex::new(SimdRegister::default()));
    // Global SIMD configuration
    static ref SIMD_CONFIG: SimdConfig =  {
        // Initalize SVE
        let mut sve_en: bool = false;
        let mut sve_vq: u64 = 0;
        let mut sme_en: bool = false;

        trace!("Reading simd features");
        #[cfg(not(any(test, miri)))]
        if ID_AA64PFR0_EL1.is_set(ID_AA64PFR0_EL1::SVE) {
            trace!("SVE is set");
            // Get effective vl
            //let _e_vl = ZCR_EL2.read(ZCR_EL2::LEN);
            // Set to maximum
            ZCR_EL2.write(ZCR_EL2::LEN.val(SVE_VQ_ARCH_MAX));
            // Get vl in bytes
            let vl_b: u64;
            unsafe {
                asm!("rdvl {}, #1", out(reg) vl_b);
            }
            sve_vq = ((vl_b << 3)/ QUARD_WORD) - 1;
            sve_en = true;
            trace!("sve_vq={:?}", sve_vq);
        }

        // init sme
        #[cfg(not(any(test, miri)))]
        if ID_AA64PFR1_SME_EL1.is_set(ID_AA64PFR1_SME_EL1::SME) {
            trace!("SME is set");
            // Find the architecturally permitted SVL
            SMCR_EL2.write(SMCR_EL2::RAZWI.val(SMCR_EL2::RAZWI.mask) + SMCR_EL2::LEN.val(SMCR_EL2::LEN.mask));
            let raz = SMCR_EL2.read(SMCR_EL2::RAZWI);
            let len = SMCR_EL2.read(SMCR_EL2::LEN);
            let sme_svq_arch_max = (raz << 4) + len;
            trace!("sme_svq_arch_max={:?}", sme_svq_arch_max);

            assert!(sme_svq_arch_max <= SVE_VQ_ARCH_MAX);
            sme_en = true;
        }

        SimdConfig {
            sve_en,
            sve_vq,
            sme_en,
        }
    };
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

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[target_feature(enable = "neon")]
unsafe fn save_fpu(fpu: &mut FpuRegs) {
    let fpsr: u64;
    let fpcr: u64;
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
        "mrs {fpsr}, fpsr",
        "mrs {fpcr}, fpcr",
        addr_q = in(reg) addr_q,
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
             "msr fpsr, {fpsr}",
             "msr fpcr, {fpcr}",
             addr_q = in(reg) addr_q,
             fpsr = in(reg) fpu.fpsr,
             fpcr = in(reg) fpu.fpcr,
        );
    }
}

pub fn restore_state(rec: &Rec<'_>) {
    let rec_simd = &rec.context.simd;
    let mut ns_simd = NS_SIMD[get_cpu_id()].lock();

    // Disable simd traps during the context mgmt.
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    if rec_simd.cfg.sve_en {
        ns_simd.sve.zcr_el2 = ZCR_EL2.get();
        ZCR_EL2.set(rec_simd.sve.zcr_el2);
    } else {
        unsafe {
            save_fpu(&mut ns_simd.fpu);
            // Don't restore rec's simd context here.
            // Restore fpu only if accessed instead.
        }
    }
    ns_simd.cptr_el2 = CPTR_EL2.get();
    CPTR_EL2.set(rec_simd.cptr_el2);
}

pub fn save_state(rec: &mut Rec<'_>) {
    let rec_simd = &mut rec.context.simd;
    let ns_simd = NS_SIMD[get_cpu_id()].lock();

    // Disable simd traps during the context mgmt.
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    if rec_simd.cfg.sve_en {
        rec_simd.sve.zcr_el2 = ZCR_EL2.get();
        rec_simd.sve.zcr_el12 = ZCR_EL1.get();
        ZCR_EL2.set(ns_simd.sve.zcr_el2);
        unimplemented!();
    } else {
        unsafe {
            if rec_simd.is_used {
                save_fpu(&mut rec_simd.fpu);
                rec_simd.is_saved = true;
            }
            restore_fpu(&ns_simd.fpu);
        }
    }
    // To maintain immutability of rec.context during its restoration,
    // update the context here in advance.
    rec_simd.is_used = false;
    rec_simd.cptr_el2 =
        (CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET + CPTR_EL2::TFP::SET + CPTR_EL2::TZ::SET).value;
    CPTR_EL2.set(ns_simd.cptr_el2);
}

pub fn validate(en: bool, sve_vl: u64) -> bool {
    if en && !SIMD_CONFIG.sve_en {
        return false;
    }
    if sve_vl > SIMD_CONFIG.sve_vq {
        return false;
    }
    true
}
