use crate::helper::ICH_VTR_EL2;
use lazy_static::lazy_static;

const ICH_LR_PRIORITY_WIDTH: u64 = 8;

#[allow(dead_code)]
pub struct GicFeatures {
    pub nr_lrs: usize,
    pub nr_aprs: usize,
    pub pri_res0_mask: u64,
    pub max_vintid: u64,
}

lazy_static! {
    pub static ref GIC_FEATURES: GicFeatures = {
        trace!("read gic features");
        let nr_lrs = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::LIST) } as usize;
        trace!("nr_lrs (LIST) {}", nr_lrs);
        let id = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::ID) };
        let max_vintid = if id == 0 {
            (1u64 << 16) - 1
        } else {
            (1u64 << 24) - 1
        };
        trace!("id {} max_vintid {}", id, max_vintid);
        let pre = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::PRE) } + 1;
        let nr_aprs = (1 << (pre - 5)) - 1;
        trace!("pre {}, nr_aprs {}", pre, nr_aprs);
        let pri = unsafe { ICH_VTR_EL2.get_masked_value(ICH_VTR_EL2::PRI) } + 1;
        let pri_res0_mask = (1u64 << (ICH_LR_PRIORITY_WIDTH - pri)) - 1;
        trace!("pri {} pri_res0_mask {}", pri, pri_res0_mask);
        GicFeatures {
            nr_lrs,
            nr_aprs,
            pri_res0_mask,
            max_vintid,
        }
    };
}
