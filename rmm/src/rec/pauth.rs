use super::Rec;

use aarch64_cpu::registers::*;

#[repr(C)]
#[derive(Default, Debug)]
pub struct PauthRegister {
    pub apiakeylo_el1: u64,
    pub apiakeyhi_el1: u64,
    pub apibkeylo_el1: u64,
    pub apibkeyhi_el1: u64,
    pub apdakeylo_el1: u64,
    pub apdakeyhi_el1: u64,
    pub apdbkeylo_el1: u64,
    pub apdbkeyhi_el1: u64,
    pub apgakeylo_el1: u64,
    pub apgakeyhi_el1: u64,
}

pub fn init_pauth(_rec: &mut Rec<'_>) {
    // Does nothing
}

pub fn restore_state(rec: &Rec<'_>) {
    unsafe {
        _restore_state(rec);
    }
}

/// # Safety
///
/// Use pauth only for (re)storing Rec's context
#[target_feature(enable = "pacg", enable = "paca")]
unsafe fn _restore_state(rec: &Rec<'_>) {
    let pauth = &rec.context.pauth;

    APIAKEYLO_EL1.set(pauth.apiakeylo_el1);
    APIAKEYHI_EL1.set(pauth.apiakeyhi_el1);
    APIBKEYLO_EL1.set(pauth.apibkeylo_el1);
    APIBKEYHI_EL1.set(pauth.apibkeyhi_el1);
    APDAKEYLO_EL1.set(pauth.apdakeylo_el1);
    APDAKEYHI_EL1.set(pauth.apdakeyhi_el1);
    APDBKEYLO_EL1.set(pauth.apdbkeylo_el1);
    APDBKEYHI_EL1.set(pauth.apdbkeyhi_el1);
    APGAKEYLO_EL1.set(pauth.apgakeylo_el1);
    APGAKEYHI_EL1.set(pauth.apgakeyhi_el1);
}

pub fn save_state(rec: &mut Rec<'_>) {
    unsafe {
        _save_state(rec);
    }
}

/// # Safety
///
/// Use pauth only for (re)storing Rec's context
#[target_feature(enable = "pacg", enable = "paca")]
unsafe fn _save_state(rec: &mut Rec<'_>) {
    let pauth = &mut rec.context.pauth;

    pauth.apiakeylo_el1 = APIAKEYLO_EL1.get();
    pauth.apiakeyhi_el1 = APIAKEYHI_EL1.get();
    pauth.apibkeylo_el1 = APIBKEYLO_EL1.get();
    pauth.apibkeyhi_el1 = APIBKEYHI_EL1.get();
    pauth.apdakeylo_el1 = APDAKEYLO_EL1.get();
    pauth.apdakeyhi_el1 = APDAKEYHI_EL1.get();
    pauth.apdbkeylo_el1 = APDBKEYLO_EL1.get();
    pauth.apdbkeyhi_el1 = APDBKEYHI_EL1.get();
    pauth.apgakeylo_el1 = APGAKEYLO_EL1.get();
    pauth.apgakeyhi_el1 = APGAKEYHI_EL1.get();
}
