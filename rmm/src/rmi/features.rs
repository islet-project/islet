use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

const S2SZ_SHIFT: usize = 0;
const S2SZ_WIDTH: usize = 8;
const S2SZ_VALUE: usize = 48;

const LPA2_SHIFT: usize = 8;
#[allow(unused)]
const LPA2_WIDTH: usize = 1;
const LPA2_VALUE: usize = 0;

const PMU_EN_SHIFT: usize = 22;
const PMU_EN_WIDTH: usize = 1;
const PMU_EN_VALUE: usize = NOT_SUPPORTED;

const PMU_NUM_CTRS_SHIFT: usize = 23;
const PMU_NUM_CTRS_WIDTH: usize = 5;
const PMU_NUM_CTRS_VALUE: usize = 0;

const HASH_SHA_256_SHIFT: usize = 28;
const HASH_SHA_256_VALUE: usize = SUPPORTED;

const HASH_SHA_512_SHIFT: usize = 29;
const HASH_SHA_512_VALUE: usize = SUPPORTED;

const NOT_SUPPORTED: usize = 0;
const SUPPORTED: usize = 1;

const FEATURE_REGISTER_0_INDEX: usize = 0;

fn extract(reg: usize, shift: usize, width: usize) -> usize {
    let mask = mask(shift, width);
    (reg << (mask.trailing_zeros())) & mask
}

fn mask(shift: usize, width: usize) -> usize {
    (!0usize >> (64usize - width)) << shift
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::FEATURES, |arg, ret, _| {
        if arg[0] != FEATURE_REGISTER_0_INDEX {
            return Ok(());
        }

        let mut feat_reg0: usize = 0;
        feat_reg0 |= S2SZ_VALUE << S2SZ_SHIFT;
        feat_reg0 |= LPA2_VALUE << LPA2_SHIFT;
        feat_reg0 |= PMU_EN_VALUE << PMU_EN_SHIFT;
        feat_reg0 |= PMU_NUM_CTRS_VALUE << PMU_NUM_CTRS_SHIFT;
        feat_reg0 |= HASH_SHA_256_VALUE << HASH_SHA_256_SHIFT;
        feat_reg0 |= HASH_SHA_512_VALUE << HASH_SHA_512_SHIFT;

        ret[1] = feat_reg0;
        debug!("rmi::FEATURES ret:{:X}", feat_reg0);
        Ok(())
    });
}

pub fn ipa_bits(feat_reg0: usize) -> usize {
    extract(feat_reg0, S2SZ_SHIFT, S2SZ_WIDTH)
}

//TODO: locate validate() in armv9a to check against AA64MMFR_EL1 register
pub fn validate(feat_reg0: usize) -> bool {
    const MIN_IPA_SIZE: usize = 32;
    let s2sz = extract(feat_reg0, S2SZ_SHIFT, S2SZ_WIDTH);
    if !(MIN_IPA_SIZE..=S2SZ_VALUE).contains(&s2sz) {
        return false;
    }

    if extract(feat_reg0, S2SZ_SHIFT, S2SZ_WIDTH) > S2SZ_VALUE {
        return false;
    }

    // TODO: Add a check for LPA2 flag with AA64MMFR_EL1 reigster after refactoring

    if extract(feat_reg0, PMU_EN_SHIFT, PMU_EN_WIDTH) == SUPPORTED
        && extract(feat_reg0, PMU_NUM_CTRS_SHIFT, PMU_NUM_CTRS_WIDTH) != PMU_NUM_CTRS_VALUE
    {
        return false;
    }

    true
}
