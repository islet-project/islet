use crate::realm::rd::Rd;
use crate::rec::Rec;
use crate::rmi::error::Error;

use aarch64_cpu::registers::*;

fn is_feat_vmid16_present() -> bool {
    #[cfg(not(any(miri, test)))]
    let ret = ID_AA64MMFR1_EL1.read(ID_AA64MMFR1_EL1::VMIDBits)
        == ID_AA64MMFR1_EL1::VMIDBits::Bits16.into();

    #[cfg(any(miri, test))]
    let ret = true;
    ret
}

pub fn prepare_vtcr(rd: &Rd) -> Result<u64, Error> {
    let s2_starting_level = rd.s2_starting_level();
    let ipa_bits = rd.ipa_bits();

    // sl0 consists of 2 bits (2^2 == 4)
    if !(s2_starting_level >= 0 && s2_starting_level <= 3) {
        return Err(Error::RmiErrorInput);
    }

    // t0sz consists of 6 bits (2^6 == 64)
    if !(ipa_bits > 0 && ipa_bits <= 64) {
        return Err(Error::RmiErrorInput);
    }

    let mut vtcr_val = VTCR_EL2::PS::PA_40B_1TB
        + VTCR_EL2::TG0::Granule4KB
        + VTCR_EL2::SH0::Inner
        + VTCR_EL2::ORGN0::NormalWBRAWA
        + VTCR_EL2::IRGN0::NormalWBRAWA
        + VTCR_EL2::NSA::NonSecurePASpace
        + VTCR_EL2::RES1.val(1); //XXX: not sure why RES1 is in default set in tf-rmm

    if is_feat_vmid16_present() {
        vtcr_val += VTCR_EL2::VS::Bits16;
    }

    let sl0_array = [
        VTCR_EL2::SL0::Granule4KBLevel0,
        VTCR_EL2::SL0::Granule4KBLevel1,
        VTCR_EL2::SL0::Granule4KBLevel2,
        VTCR_EL2::SL0::Granule4KBLevel3,
    ];
    let sl0_val = sl0_array[s2_starting_level as usize];
    let t0sz_val = (64 - ipa_bits) as u64;

    vtcr_val += sl0_val + VTCR_EL2::T0SZ.val(t0sz_val);

    Ok(vtcr_val.into())
}

pub fn activate_stage2_mmu(rec: &Rec<'_>) {
    VTCR_EL2.set(rec.vtcr());
}
