//!  Performance Monitors Control Register - EL0

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub PMCR_EL0 [
        /// Number of event counters implemented
        N OFFSET(11) NUMBITS(5) [],
        LP OFFSET(7) NUMBITS(1) [],
        LC OFFSET(6) NUMBITS(1) [],
        DP OFFSET(5) NUMBITS(1) [],
        X OFFSET(4) NUMBITS(1) [],
        D OFFSET(3) NUMBITS(1) [],
        C OFFSET(2) NUMBITS(1) [],
        P OFFSET(1) NUMBITS(1) [],
        E OFFSET(0) NUMBITS(1) [],
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = PMCR_EL0::Register;

    sys_coproc_read_raw!(u64, "PMCR_EL0", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = PMCR_EL0::Register;

    sys_coproc_write_raw!(u64, "PMCR_EL0", "x");
}

pub const PMCR_EL0: Reg = Reg {};
