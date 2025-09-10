// SPDX-License-Identifier: Apache-2.0 OR MIT
//

//! AArch64 Processor Feature Register 1 - EL1
//!
//! Provides additional information about implemented PE features in AArch64 state.

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub SVCR [
        /// Enables SME ZA storage.
        ZA OFFSET(1) NUMBITS(1) [],
        /// Enables Streaming SVE mode.
        SM OFFSET(0) NUMBITS(1) [],
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = SVCR::Register;

    sys_coproc_read_raw!(u64, "SVCR", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = SVCR::Register;

    sys_coproc_write_raw!(u64, "SVCR", "x");
}
pub const SVCR: Reg = Reg {};
