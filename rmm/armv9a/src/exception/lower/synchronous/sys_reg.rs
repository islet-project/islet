use crate::exception::trap;
use crate::helper::bits_in_reg;
use crate::realm::context::Context;
use crate::{define_bitfield, define_bits, define_mask, define_sys_register};

use monitor::realm::vcpu::VCPU;

macro_rules! define_iss_id {
    ($name:ident, $Op0:expr, $Op1:expr, $CRn:expr, $CRm:expr, $Op2:expr) => {
        const $name: u32 = bits_in_reg(ISS::Op0, $Op0) as u32
            | bits_in_reg(ISS::Op1, $Op1) as u32
            | bits_in_reg(ISS::CRn, $CRn) as u32
            | bits_in_reg(ISS::CRm, $CRm) as u32
            | bits_in_reg(ISS::Op2, $Op2) as u32;
    };
}

define_bits!(
    ISS,
    IL[25 - 25],
    Op0[21 - 20],
    Op2[19 - 17],
    Op1[16 - 14],
    CRn[13 - 10],
    Rt[9 - 5],
    CRm[4 - 1],
    Direction[0 - 0]
);

define_sys_register!(ID_AA64PFR0_EL1);
define_bits!(AA64PFR0, AMU[47 - 44], SVE[35 - 32]);
define_iss_id!(ISS_ID_AA64PFR0_EL1, 3, 0, 0, 4, 0);

define_sys_register!(ID_AA64PFR1_EL1);
define_iss_id!(ISS_ID_AA64PFR1_EL1, 3, 0, 0, 4, 1);

// TODO: current compiler doesn't understand this sysreg
//define_sys_register!(ID_AA64ZFR0_EL1);
//define_iss_id!(ISS_ID_AA64ZFR0_EL1, 3, 0, 0, 4, 4);

define_sys_register!(ID_AA64DFR0_EL1);
define_iss_id!(ISS_ID_AA64DFR0_EL1, 3, 0, 0, 5, 0);

define_sys_register!(ID_AA64DFR1_EL1);
define_iss_id!(ISS_ID_AA64DFR1_EL1, 3, 0, 0, 5, 1);

define_sys_register!(ID_AA64AFR0_EL1);
define_iss_id!(ISS_ID_AA64AFR0_EL1, 3, 0, 0, 5, 4);

define_sys_register!(ID_AA64AFR1_EL1);
define_iss_id!(ISS_ID_AA64AFR1_EL1, 3, 0, 0, 5, 5);

define_sys_register!(ID_AA64ISAR0_EL1);
define_iss_id!(ISS_ID_AA64ISAR0_EL1, 3, 0, 0, 6, 0);

define_sys_register!(ID_AA64ISAR1_EL1);
define_bits!(
    AA64ISAR1,
    GPI[31 - 28],
    GPA[27 - 24],
    APA[7 - 4],
    API[11 - 8]
);
define_iss_id!(ISS_ID_AA64ISAR1_EL1, 3, 0, 0, 6, 1);

define_sys_register!(ID_AA64MMFR0_EL1);
define_iss_id!(ISS_ID_AA64MMFR0_EL1, 3, 0, 0, 7, 0);

define_sys_register!(ID_AA64MMFR1_EL1);
define_iss_id!(ISS_ID_AA64MMFR1_EL1, 3, 0, 0, 7, 1);

define_sys_register!(ID_AA64MMFR2_EL1);
define_iss_id!(ISS_ID_AA64MMFR2_EL1, 3, 0, 0, 7, 2);

fn check_sysreg_id_access(esr: u64) -> bool {
    let esr = ISS::new(esr);
    (esr.get_masked(ISS::Op0) | esr.get_masked(ISS::Op1) | esr.get_masked(ISS::CRn)) == ISS::Op0
}

pub fn handle(vcpu: &mut VCPU<Context>, esr: u64) -> u64 {
    if check_sysreg_id_access(esr) {
        handle_sysreg_id(vcpu, esr);
    }
    trap::RET_TO_REC
}

fn handle_sysreg_id(vcpu: &mut VCPU<Context>, esr: u64) -> u64 {
    let esr = ISS::new(esr);
    let il = esr.get_masked_value(ISS::IL);
    let rt = esr.get_masked_value(ISS::Rt) as usize;
    // direction: 0b0 - write, 0b1 - read
    let direction = esr.get_masked_value(ISS::Direction);

    if il == 0 {
        error!("Exception taken from 32bit arch. Realm needs to be 64bit(arm64).");
    }
    if direction == 0 {
        warn!("Unable to write id system reg. Will ignore this request!");
        return trap::RET_TO_REC;
    }
    if rt == 31 {
        trace!("handle_sysreg_id(): Rt = xzr");
        return trap::RET_TO_REC;
    }

    let idreg = esr.get_masked(ISS::Op0)
        | esr.get_masked(ISS::Op1)
        | esr.get_masked(ISS::CRn)
        | esr.get_masked(ISS::CRm)
        | esr.get_masked(ISS::Op2);

    let mut mask: u64 = match idreg as u32 {
        ISS_ID_AA64ISAR1_EL1 => AA64ISAR1::GPI | AA64ISAR1::GPA | AA64ISAR1::APA | AA64ISAR1::API,
        ISS_ID_AA64PFR0_EL1 => AA64PFR0::AMU | AA64PFR0::SVE,
        _ => 0,
    };
    mask = !mask;

    vcpu.context.gp_regs[rt] = match idreg as u32 {
        ISS_ID_AA64PFR0_EL1 => unsafe { ID_AA64PFR0_EL1.get() & mask },
        ISS_ID_AA64PFR1_EL1 => unsafe { ID_AA64PFR1_EL1.get() & mask },
        //ISS_ID_AA64ZFR0_EL1 => unsafe { ID_AA64ZFR0_EL1.get()  & mask },
        ISS_ID_AA64DFR0_EL1 => unsafe { ID_AA64DFR0_EL1.get() & mask },
        ISS_ID_AA64DFR1_EL1 => unsafe { ID_AA64DFR1_EL1.get() & mask },
        ISS_ID_AA64AFR0_EL1 => unsafe { ID_AA64AFR0_EL1.get() & mask },
        ISS_ID_AA64AFR1_EL1 => unsafe { ID_AA64AFR1_EL1.get() & mask },
        ISS_ID_AA64ISAR0_EL1 => unsafe { ID_AA64ISAR0_EL1.get() & mask },
        ISS_ID_AA64ISAR1_EL1 => unsafe { ID_AA64ISAR1_EL1.get() & mask },
        ISS_ID_AA64MMFR0_EL1 => unsafe { ID_AA64MMFR0_EL1.get() & mask },
        ISS_ID_AA64MMFR1_EL1 => unsafe { ID_AA64MMFR1_EL1.get() & mask },
        ISS_ID_AA64MMFR2_EL1 => unsafe { ID_AA64MMFR2_EL1.get() & mask },
        _ => 0x0,
    };
    trap::RET_TO_REC
}
