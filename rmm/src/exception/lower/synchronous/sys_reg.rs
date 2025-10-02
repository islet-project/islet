use crate::exception::trap;
use crate::rec::Rec;

use aarch64_cpu::registers::*;
use armv9a::regs::*;

fn check_sysreg_id_access(esr: u64) -> bool {
    let esr = ISS::new(esr);
    (esr.get_masked(ISS::Op0) | esr.get_masked(ISS::Op1) | esr.get_masked(ISS::CRn)) == ISS::Op0
}

fn check_sysreg_icc_access(esr: u64) -> bool {
    let esr = ISS::new(esr);
    // direction: 0b0 - write, 0b1 - read
    let direction = esr.get_masked_value(ISS::Direction);
    let esr_iss = esr.get_masked(ISS::Op0 | ISS::Op1 | ISS::CRn | ISS::CRm) as u32;

    // Only writing to the system register is valid.
    direction == 0 && (esr_iss == ISS_ID_ICC_MASK || esr_iss == ISS_ID_ICC_PMR_EL1)
}

pub fn handle(rec: &mut Rec<'_>, esr: u64) -> u64 {
    if check_sysreg_id_access(esr) {
        handle_sysreg_id(rec, esr);
    } else if check_sysreg_icc_access(esr) {
        return trap::RET_TO_RMM;
    } else {
        warn!("Unhandled MSR/MRS instruction. ESR_EL2:{:X}", esr);
    }
    trap::RET_TO_REC
}

fn handle_sysreg_id(rec: &mut Rec<'_>, esr: u64) -> u64 {
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
        ISS_ID_AA64PFR0_EL1 => {
            (ID_AA64PFR0_EL1::AMU.mask << ID_AA64PFR0_EL1::AMU.shift)
                + if !rec.context.simd.cfg.sve_en {
                    ID_AA64PFR0_EL1::SVE.mask << ID_AA64PFR0_EL1::SVE.shift
                } else {
                    0
                }
        }
        // Present FEAT_SVE only if Rec is set to use SVE.
        ISS_ID_AA64ZFR0_EL1 => (!rec.context.simd.cfg.sve_en as u64).wrapping_neg(),
        ISS_ID_AA64PFR1_EL1 => {
            (ID_AA64PFR1_SME_EL1::MTE.mask << ID_AA64PFR1_SME_EL1::MTE.shift)
                + (ID_AA64PFR1_SME_EL1::SME.mask << ID_AA64PFR1_SME_EL1::SME.shift)
        }
        ISS_ID_AA64DFR0_EL1 => {
            (ID_AA64DFR0_EL1::BRBE.mask << ID_AA64DFR0_EL1::BRBE.shift)
                + (ID_AA64DFR0_EL1::MTPMU.mask << ID_AA64DFR0_EL1::MTPMU.shift)
                + (ID_AA64DFR0_EL1::TraceBuffer.mask << ID_AA64DFR0_EL1::TraceBuffer.shift)
                + (ID_AA64DFR0_EL1::TraceFilt.mask << ID_AA64DFR0_EL1::TraceFilt.shift)
                + (ID_AA64DFR0_EL1::PMSVer.mask << ID_AA64DFR0_EL1::PMSVer.shift)
                + (ID_AA64DFR0_EL1::CTX_CMPs.mask << ID_AA64DFR0_EL1::CTX_CMPs.shift)
                + (ID_AA64DFR0_EL1::WRPs.mask << ID_AA64DFR0_EL1::WRPs.shift)
                + (ID_AA64DFR0_EL1::BRPs.mask << ID_AA64DFR0_EL1::BRPs.shift)
                + (ID_AA64DFR0_EL1::PMUVer.mask << ID_AA64DFR0_EL1::PMUVer.shift)
                + (ID_AA64DFR0_EL1::TraceVer.mask << ID_AA64DFR0_EL1::TraceVer.shift)
                + (ID_AA64DFR0_EL1::DebugVer.mask << ID_AA64DFR0_EL1::DebugVer.shift)
        }
        _ => 0,
    };
    mask = !mask;

    rec.context.gp_regs[rt] = match idreg as u32 {
        ISS_ID_AA64PFR0_EL1 => ID_AA64PFR0_EL1.get() & mask,
        ISS_ID_AA64PFR1_EL1 => ID_AA64PFR1_EL1.get() & mask,
        ISS_ID_AA64ZFR0_EL1 => ID_AA64ZFR0_EL1.get() & mask,
        ISS_ID_AA64DFR0_EL1 => {
            let mut dfr0_set = 0u64;
            dfr0_set &= 6 << ID_AA64DFR0_EL1::DebugVer.shift;
            dfr0_set &= 1 << ID_AA64DFR0_EL1::BRPs.shift;
            dfr0_set &= 1 << ID_AA64DFR0_EL1::WRPs.shift;
            ID_AA64DFR0_EL1.get() & mask | dfr0_set
        }
        ISS_ID_AA64DFR1_EL1 => ID_AA64DFR1_EL1.get() & mask,
        ISS_ID_AA64AFR0_EL1 => ID_AA64AFR0_EL1.get(),
        ISS_ID_AA64AFR1_EL1 => ID_AA64AFR1_EL1.get(),
        ISS_ID_AA64ISAR0_EL1 => ID_AA64ISAR0_EL1.get(),
        ISS_ID_AA64ISAR1_EL1 => ID_AA64ISAR1_EL1.get(),
        ISS_ID_AA64MMFR0_EL1 => ID_AA64MMFR0_EL1.get(),
        ISS_ID_AA64MMFR1_EL1 => ID_AA64MMFR1_EL1.get(),
        ISS_ID_AA64MMFR2_EL1 => ID_AA64MMFR2_EL1.get(), //0x10211122,
        _ => 0x0,
    };
    trap::RET_TO_REC
}
