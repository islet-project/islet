use crate::exception::trap;
use crate::realm::context::Context;
use crate::realm::vcpu::VCPU;

use armv9a::regs::*;

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
        ISS_ID_AA64PFR1_EL1 => AA64PFR1::MTE,
        ISS_ID_AA64DFR0_EL1 => {
            AA64DFR0::BRBE
                | AA64DFR0::MTPMU
                | AA64DFR0::TraceBuffer
                | AA64DFR0::TraceFilt
                | AA64DFR0::PMSVer
                | AA64DFR0::CTX_CMPs
                | AA64DFR0::WRPs
                | AA64DFR0::BRPs
                | AA64DFR0::PMUVer
                | AA64DFR0::TraceVer
                | AA64DFR0::DebugVer
        }
        _ => 0,
    };
    mask = !mask;

    vcpu.context.gp_regs[rt] = match idreg as u32 {
        ISS_ID_AA64PFR0_EL1 => unsafe { ID_AA64PFR0_EL1.get() & mask },
        ISS_ID_AA64PFR1_EL1 => unsafe { ID_AA64PFR1_EL1.get() & mask },
        //ISS_ID_AA64ZFR0_EL1 => unsafe { ID_AA64ZFR0_EL1.get()  & mask },
        ISS_ID_AA64DFR0_EL1 => unsafe {
            let mut dfr0_set = AA64DFR0(0);
            dfr0_set.set_masked_value(AA64DFR0::DebugVer, 6);
            dfr0_set.set_masked_value(AA64DFR0::BRPs, 1);
            dfr0_set.set_masked_value(AA64DFR0::WRPs, 1);
            ID_AA64DFR0_EL1.get() & mask | dfr0_set.get()
        },
        ISS_ID_AA64DFR1_EL1 => unsafe { ID_AA64DFR1_EL1.get() & mask },
        ISS_ID_AA64AFR0_EL1 => unsafe { ID_AA64AFR0_EL1.get() & mask },
        ISS_ID_AA64AFR1_EL1 => unsafe { ID_AA64AFR1_EL1.get() & mask },
        ISS_ID_AA64ISAR0_EL1 => unsafe { ID_AA64ISAR0_EL1.get() & mask },
        ISS_ID_AA64ISAR1_EL1 => unsafe { ID_AA64ISAR1_EL1.get() & mask },
        ISS_ID_AA64MMFR0_EL1 => unsafe { ID_AA64MMFR0_EL1.get() & mask },
        ISS_ID_AA64MMFR1_EL1 => unsafe { ID_AA64MMFR1_EL1.get() & mask },
        ISS_ID_AA64MMFR2_EL1 => unsafe { ID_AA64MMFR2_EL1.get() & mask }, //0x10211122,
        _ => 0x0,
    };
    trap::RET_TO_REC
}
