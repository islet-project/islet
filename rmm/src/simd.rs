use aarch64_cpu::registers::ID_AA64PFR0_EL1;
use aarch64_cpu::registers::{Readable, Writeable};
use armv9a::regs::{ID_AA64PFR1_SME_EL1, SMCR_EL2, ZCR_EL2};
use core::arch::asm;
use lazy_static::lazy_static;

// Vector length (VL) = size of a Z-register in bytes
// Vector quadwords (VQ) = size of a Z-register in units of 128 bits
// Minimum length of a SVE vector: 128 bits
pub const ZCR_EL2_LEN_WIDTH: u64 = 4;
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

lazy_static! {
    // Global SIMD configuration
    static ref SIMD_CONFIG: SimdConfig =  {
        // Initalize SVE
        let mut sve_en: bool = false;
        let mut sve_vq: u64 = 0;
        let mut sme_en: bool = false;

        trace!("Reading simd features");
        #[cfg(not(any(test, miri, fuzzing)))]
        if ID_AA64PFR0_EL1.is_set(ID_AA64PFR0_EL1::SVE) {
            trace!("SVE is set");
            // Get effective vl: (ZCR_EL2:LEN + 1)*128 bits
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
        #[cfg(not(any(test, miri, fuzzing)))]
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
