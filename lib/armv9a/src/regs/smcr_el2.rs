//! Realm Management Monitor Configuration Register - EL2

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub SMCR_EL2 [
        RAZWI   OFFSET(4) NUMBITS(5) [],

        LEN OFFSET(0) NUMBITS(4) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = SMCR_EL2::Register;

    sys_coproc_read_raw!(u64, "SMCR_EL2", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = SMCR_EL2::Register;

    sys_coproc_write_raw!(u64, "SMCR_EL2", "x");
}

pub const SMCR_EL2: Reg = Reg {};
