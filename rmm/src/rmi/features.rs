use crate::event::RmiHandle;
use crate::listen;
use crate::rec;
use crate::rec::gic;
use crate::rmi;
use crate::simd;
use armv9a::{define_bitfield, define_bits, define_mask};

extern crate alloc;

define_bits!(
    FeatureReg0,
    MAX_RECS_ORDER[41 - 38],
    GICV3_NUM_LRS[37 - 34],
    HASH_SHA_512[33 - 33],
    HASH_SHA_256[32 - 32],
    PMU_NUM_CTRS[31 - 27],
    PMU_EN[26 - 26],
    NUM_WPS[25 - 20],
    NUM_BPS[19 - 14],
    SVE_VL[13 - 10],
    SVE_EN[9 - 9],
    LPA2[8 - 8],
    S2SZ[7 - 0]
);

const S2SZ_VALUE: u64 = 48;
pub const LPA2_VALUE: u64 = 0;
pub const PMU_EN_VALUE: u64 = NOT_SUPPORTED;
const PMU_NUM_CTRS_VALUE: u64 = 0;
const HASH_SHA_256_VALUE: u64 = SUPPORTED;
const HASH_SHA_512_VALUE: u64 = SUPPORTED;

const NOT_SUPPORTED: u64 = 0;
const SUPPORTED: u64 = 1;

const FEATURE_REGISTER_0_INDEX: usize = 0;

pub fn set_event_handler(rmi: &mut RmiHandle) {
    listen!(rmi, rmi::FEATURES, |arg, ret, _| {
        if arg[0] != FEATURE_REGISTER_0_INDEX {
            ret[1] = 0;
            return Ok(());
        }

        let mut feat_reg0 = FeatureReg0::new(0);
        #[cfg(not(any(miri, test)))]
        feat_reg0
            .set_masked_value(FeatureReg0::S2SZ, S2SZ_VALUE)
            .set_masked_value(FeatureReg0::LPA2, LPA2_VALUE)
            .set_masked_value(
                FeatureReg0::SVE_EN,
                if simd::sve_en() {
                    SUPPORTED
                } else {
                    NOT_SUPPORTED
                },
            )
            .set_masked_value(FeatureReg0::SVE_VL, simd::max_sve_vl())
            .set_masked_value(FeatureReg0::PMU_EN, PMU_EN_VALUE)
            .set_masked_value(FeatureReg0::PMU_NUM_CTRS, PMU_NUM_CTRS_VALUE)
            .set_masked_value(FeatureReg0::HASH_SHA_256, HASH_SHA_256_VALUE)
            .set_masked_value(FeatureReg0::HASH_SHA_512, HASH_SHA_512_VALUE)
            .set_masked_value(FeatureReg0::GICV3_NUM_LRS, gic::nr_lrs() as u64)
            .set_masked_value(FeatureReg0::MAX_RECS_ORDER, rec::max_recs_order() as u64);

        ret[1] = feat_reg0.get() as usize;
        debug!("rmi::FEATURES ret:{:X}", feat_reg0.get());
        Ok(())
    });
}

//TODO: locate validate() in armv9a to check against AA64MMFR_EL1 register
pub fn validate(s2sz: usize) -> bool {
    const MIN_IPA_SIZE: usize = 32;
    if !(MIN_IPA_SIZE..=S2SZ_VALUE as usize).contains(&s2sz) {
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    use crate::rmi::{FEATURES, SUCCESS};
    use crate::test_utils::*;

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rmi_features_host
    #[test]
    fn rmi_features() {
        let ret = rmi::<FEATURES>(&[0]);

        assert_eq!(ret[0], SUCCESS);
        assert_eq!(extract_bits(ret[1], 30, 63), 0);

        let ret = rmi::<FEATURES>(&[1]);
        assert_eq!(ret[0], SUCCESS);
        assert_eq!(ret[1], 0);
    }
}
