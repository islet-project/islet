defreg!(CurrentEL, EL[3 - 2]);
pub unsafe fn current_el() -> u8 {
    CurrentEL.get_masked_value(CurrentEL::EL) as u8
}

defreg!(VBAR_EL2, RES0[10 - 0]);
defreg!(
    ESR_EL2,
    EC[31 - 26],
    IL[25 - 25],
    ISS[24 - 00],
    ISS_BRK_CMT[15 - 00],
    DFSC[5 - 0]
);
