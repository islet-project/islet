//! Realm Management Monitor Configuration Register - EL2

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub MDCR_EL2 [
        MTPME OFFSET(28) NUMBITS(1) [],
        HCCD OFFSET(23) NUMBITS(1) [],
        HPMD OFFSET(17) NUMBITS(1) [],
        TDA OFFSET(9) NUMBITS(1) [],
        TPM OFFSET(6) NUMBITS(1) [],
        TPMCR OFFSET(5) NUMBITS(1) [],
        HPMN OFFSET(0) NUMBITS(5) [],
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = MDCR_EL2::Register;

    sys_coproc_read_raw!(u64, "MDCR_EL2", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = MDCR_EL2::Register;

    sys_coproc_write_raw!(u64, "MDCR_EL2", "x");
}

pub const MDCR_EL2: Reg = Reg {};
