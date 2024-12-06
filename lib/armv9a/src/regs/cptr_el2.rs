use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub CPTR_EL2 [
        // Trap accesses to CPACR_EL1 from EL1 to EL2,
        TCPAC  OFFSET(31) NUMBITS(1) [],
        // Trap Activity Monitor access from EL1 and EL0.
        TAM  OFFSET(30) NUMBITS(1) [],
        // Traps System register accesses to all implemented trace registers
        TTA OFFSET(20) NUMBITS(1) [],
        // Traps execution of SME instructions
        TSM OFFSET(12) NUMBITS(1) [],
        // Traps execution of Advanced SIMD and floating-point instructions
        TFP OFFSET(10) NUMBITS(1) [],
        // Traps execution of SVE instructions
        TZ OFFSET(8) NUMBITS(1) [],
/*
        // Traps System register accesses to all implemented trace registers
        TTA OFFSET(28) NUMBITS(1) [],
        // Traps execution at EL2, EL1, and EL0 of SME instructions
        SMEN OFFSET(24) NUMBITS(2) [
            TrapAll = 0b00,
            TrapE0 = 0b01,
            TrapNone = 0b11,
        ],
        // Traps execution of SIMD and FPU
        FPEN OFFSET(20) NUMBITS(2) [
            TrapAll = 0b00,
            TrapE0 = 0b01,
            TrapNone = 0b11,
        ],
        // Traps execution of non-streaming SVE
        ZEN OFFSET(16) NUMBITS(2) [
            TrapAll = 0b00,
            TrapE0 = 0b01,
            TrapNone = 0b11,
        ]
*/
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = CPTR_EL2::Register;

    sys_coproc_read_raw!(u64, "CPTR_EL2", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = CPTR_EL2::Register;

    sys_coproc_write_raw!(u64, "CPTR_EL2", "x");
}

pub const CPTR_EL2: Reg = Reg {};
