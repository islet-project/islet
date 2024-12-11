//! Realm Management Monitor Configuration Register - EL2

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub ZCR_EL2 [
        RAZWI   OFFSET(4) NUMBITS(5) [],

        LEN OFFSET(0) NUMBITS(4) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = ZCR_EL2::Register;

    //sys_coproc_read_raw!(u64, "ZCR_EL2", "x");
    sys_coproc_read_raw!(u64, "S3_4_C1_C2_0", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = ZCR_EL2::Register;

    //sys_coproc_write_raw!(u64, "ZCR_EL2", "x");
    sys_coproc_write_raw!(u64, "S3_4_C1_C2_0", "x");
}

pub const ZCR_EL2: Reg = Reg {};
