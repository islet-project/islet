use crate::asm::isb;
use aarch64_cpu::registers::ID_AA64PFR0_EL1;
use aarch64_cpu::registers::{Readable, Writeable};
use armv9a::regs::{CPTR_EL2, ID_AA64PFR1_SME_EL1, SMCR_EL2, ZCR_EL2};
use core::arch::asm;
use lazy_static::lazy_static;

// Vector length (VL) = size of a Z-register in bytes
// Vector quadwords (VQ) = size of a Z-register in units of 128 bits
// Minimum length of a SVE vector: 128 bits
const ZCR_EL2_LEN_WIDTH: u64 = 4;
const SVE_VQ_ARCH_MAX: u64 = (1 << ZCR_EL2_LEN_WIDTH) - 1;
const QUARD_WORD: u64 = 128;
// Note: Limit maximun vq to 4 to avoid exceeding a page boundary.
// Currunt rmm implmentation maps a physical page to a virtual address
// using its physical address (i.e. identical mapping).
// Discontiguous Aux granules cannot not be accessed contiguously
// in their virtual address. For this reason, limit vq up to 4(~2184 bytes).
// Z regs = 16bytes * 32 regs * 4 vq
// P regs = 2 bytes * 16 regs * 4 vq
// FFR reg = 2 bytes * 4 vq
pub const MAX_VQ: u64 = 4;

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

lazy_static! {
    // Global SIMD configuration
    static ref SIMD_CONFIG: SimdConfig =  {
        // Initalize SVE
        let mut sve_en: bool = false;
        let mut sve_vq: u64 = 0;
        let mut sme_en: bool = false;

        trace!("Reading simd features");
        #[cfg(not(any(test, miri, fuzzing)))]
        {
            // Unlike TF-RMM (which probes SVE/SME once during RMM cold boot before
            // the Non-Secure host starts), `SIMD_CONFIG` is a lazy static evaluated
            // on the first RMI call that queries it (e.g. `RMI_FEATURES`), when the
            // NS host is already running.
            //
            // Because EL3 (TF-A) does not bank `ZCR_EL2` or `SMCR_EL2` in its per-world
            // `el2_sysregs` context, those registers currently hold the live NS-EL2
            // values (such as NS's configured vector length caps and `SMCR_EL2.FA64`).
            // Therefore, we must save and restore `ZCR_EL2` and `SMCR_EL2` around the
            // destructive max-VL/SVL probes below so we do not clobber NS-EL2 state.
            let orig_cptr = CPTR_EL2.get();
            CPTR_EL2.write(CPTR_EL2::TAM::SET);
            isb();

            if ID_AA64PFR0_EL1.is_set(ID_AA64PFR0_EL1::SVE) {
                trace!("SVE is set");
                // Preserve NS-EL2's live ZCR_EL2 before writing SVE_VQ_ARCH_MAX to probe VL.
                let orig_zcr = ZCR_EL2.get();
                // Get effective vl: (ZCR_EL2:LEN + 1)*128 bits
                // Set to maximum
                ZCR_EL2.write(ZCR_EL2::LEN.val(SVE_VQ_ARCH_MAX));
                isb();
                // Get vl in bytes
                let vl_b = unsafe { get_vector_length_bytes() };
                ZCR_EL2.set(orig_zcr);
                isb();
                sve_vq = ((vl_b << 3) / QUARD_WORD) - 1;
                if sve_vq > MAX_VQ {
                    sve_vq = MAX_VQ - 1;
                }
                sve_en = true;
                trace!("sve_vq={:?}", sve_vq);
            }

            // init sme
            if ID_AA64PFR1_SME_EL1.is_set(ID_AA64PFR1_SME_EL1::SME) {
                trace!("SME is set");
                // Preserve NS-EL2's live SMCR_EL2 (including LEN, FA64, EZT0) before
                // overwriting it to probe the maximum architecturally permitted SVL.
                let orig_smcr = SMCR_EL2.get();
                // Find the architecturally permitted SVL
                SMCR_EL2.write(
                    SMCR_EL2::RAZWI.val(SMCR_EL2::RAZWI.mask)
                        + SMCR_EL2::LEN.val(SMCR_EL2::LEN.mask),
                );
                isb();
                let raz = SMCR_EL2.read(SMCR_EL2::RAZWI);
                let len = SMCR_EL2.read(SMCR_EL2::LEN);
                SMCR_EL2.set(orig_smcr);
                isb();
                let sme_svq_arch_max = (raz << 4) + len;
                trace!("sme_svq_arch_max={:?}", sme_svq_arch_max);

                assert!(sme_svq_arch_max <= SVE_VQ_ARCH_MAX);
                sme_en = true;
            }

            CPTR_EL2.set(orig_cptr);
            isb();
        }

        SimdConfig {
            sve_en,
            sve_vq,
            sme_en,
        }
    };
}

/// Get the SVE vector length in bytes using the RDVL instruction
#[allow(aarch64_softfloat_neon)]
#[target_feature(enable = "sve")]
unsafe fn get_vector_length_bytes() -> u64 {
    let vl_b: u64;
    unsafe {
        asm!("rdvl {}, #1", out(reg) vl_b);
    }
    vl_b
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

pub fn sve_en() -> bool {
    SIMD_CONFIG.sve_en
}

pub fn max_sve_vl() -> u64 {
    SIMD_CONFIG.sve_vq
}

pub fn sme_en() -> bool {
    SIMD_CONFIG.sme_en
}
