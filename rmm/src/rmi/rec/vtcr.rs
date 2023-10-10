use super::Rec;
use crate::rmi::error::Error;
use crate::rmi::realm::Rd;
use armv9a::bits_in_reg;
use armv9a::regs::*;

fn is_feat_vmid16_present() -> bool {
    unsafe { ID_AA64MMFR1_EL1.get_masked_value(ID_AA64MMFR1_EL1::VMID) == mmfr1_vmid::VMIDBITS_16 }
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

    let mut vtcr_val = bits_in_reg(VTCR_EL2::PS, tcr_paddr_size::PS_1T)
        | bits_in_reg(VTCR_EL2::TG0, tcr_granule::G_4K)
        | bits_in_reg(VTCR_EL2::SH0, tcr_shareable::INNER)
        | bits_in_reg(VTCR_EL2::ORGN0, tcr_cacheable::WBWA)
        | bits_in_reg(VTCR_EL2::IRGN0, tcr_cacheable::WBWA)
        | bits_in_reg(VTCR_EL2::NSA, 1)
        | bits_in_reg(VTCR_EL2::RES1, 1); //XXX: not sure why RES1 is in default set in tf-rmm

    if is_feat_vmid16_present() {
        vtcr_val |= bits_in_reg(VTCR_EL2::VS, 1);
    }

    let sl0_array: [u64; 4] = [
        vtcr_sl0::SL0_4K_L0,
        vtcr_sl0::SL0_4K_L1,
        vtcr_sl0::SL0_4K_L2,
        vtcr_sl0::SL0_4K_L3,
    ];
    let sl0_val = sl0_array[s2_starting_level as usize];
    let t0sz_val = (64 - ipa_bits) as u64;

    vtcr_val |= bits_in_reg(VTCR_EL2::SL0, sl0_val) | bits_in_reg(VTCR_EL2::T0SZ, t0sz_val);

    Ok(vtcr_val)
}

pub fn activate_stage2_mmu(rec: &Rec) {
    unsafe {
        VTCR_EL2.set(rec.vtcr());
    }
}
