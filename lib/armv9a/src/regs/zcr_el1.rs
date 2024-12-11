//! Realm Management Monitor Configuration Register - EL1

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub ZCR_EL1 [
        RAZWI   OFFSET(4) NUMBITS(5) [],

        LEN OFFSET(0) NUMBITS(4) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = ZCR_EL1::Register;

    //sys_coproc_read_raw!(u64, "ZCR_EL1", "x");
    sys_coproc_read_raw!(u64, "S3_0_C1_C2_0", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = ZCR_EL1::Register;

    //sys_coproc_write_raw!(u64, "ZCR_EL1", "x");
    sys_coproc_write_raw!(u64, "S3_0_C1_C2_0", "x");
}

pub const ZCR_EL1: Reg = Reg {};
