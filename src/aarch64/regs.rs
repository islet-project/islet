define_sys_register!(CurrentEL, EL[3 - 2]);
pub fn current_el() -> u8 {
    unsafe { CurrentEL.get_masked_value(CurrentEL::EL) as u8 }
}

define_sys_register!(VBAR_EL2, RES0[10 - 0]);

define_sys_register!(
    ESR_EL2,
    EC[31 - 26],
    IL[25 - 25],
    ISS[24 - 00],
    ISS_BRK_CMT[15 - 00],
    DFSC[5 - 0]
);

define_register!(SP);
define_sys_register!(SP_EL0);
define_sys_register!(SP_EL1);
define_sys_register!(SP_EL2);

define_sys_register!(
    SPSR_EL2,
    D[9 - 9], // Debug exception mask
    A[8 - 8], // SError exception mask
    I[7 - 7], // IRQ exception mask
    F[6 - 6], // FIQ exception mask
    M[3 - 0]  // Exception level and selected SP
);

define_sys_register!(ELR_EL2);
