use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

const S2SZ_SHIFT: usize = 0;
const S2SZ_VALUE: usize = 48;

const LPA2_SHIFT: usize = 8;
const LPA2_VALUE: usize = 0;

const PMU_EN_SHIFT: usize = 22;
const PMU_EN_VALUE: usize = NOT_SUPPORTED;

const PMU_NUM_CTRS_SHIFT: usize = 23;
const PMU_NUM_CTRS_VALUE: usize = 0;

const HASH_SHA_256_SHIFT: usize = 28;
const HASH_SHA_256_VALUE: usize = SUPPORTED;

const HASH_SHA_512_SHIFT: usize = 29;
const HASH_SHA_512_VALUE: usize = SUPPORTED;

const NOT_SUPPORTED: usize = 0;
const SUPPORTED: usize = 1;

const FEATURE_REGISTER_0_INDEX: usize = 0;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::FEATURES, |ctx, _| {
        if ctx.arg[0] != FEATURE_REGISTER_0_INDEX {
            ctx.ret[0] = rmi::ERROR_INPUT;
            return;
        }

        let mut feat_reg0: usize = 0;
        feat_reg0 |= S2SZ_VALUE << S2SZ_SHIFT;
        feat_reg0 |= LPA2_VALUE << LPA2_SHIFT;
        feat_reg0 |= PMU_EN_VALUE << PMU_EN_SHIFT;
        feat_reg0 |= PMU_NUM_CTRS_VALUE << PMU_NUM_CTRS_SHIFT;
        feat_reg0 |= HASH_SHA_256_VALUE << HASH_SHA_256_SHIFT;
        feat_reg0 |= HASH_SHA_512_VALUE << HASH_SHA_512_SHIFT;

        ctx.ret[0] = rmi::SUCCESS;
        ctx.ret[1] = feat_reg0;
        debug!("rmi::FEATURES ret:{:X}", feat_reg0);
    });
}
