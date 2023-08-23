pub mod constraint;
pub mod hostcall;
pub mod psci;

use crate::define_interface;
use crate::event::RsiHandle;
use crate::listen;
use crate::rmi;
use crate::rsi::hostcall::HostCall;

define_interface! {
    command {
        ABI_VERSION= 0xc400_0190,
        MEASUREMENT_READ= 0xc400_0192,
        MEASUREMENT_EXTEND= 0xc400_0193,
        ATTEST_TOKEN_INIT= 0xc400_0194,
        ATTEST_TOKEN_CONTINUE= 0xc400_0195,
        REALM_CONFIG= 0xc400_0196,
        IPA_STATE_SET= 0xc400_0197,
        HOST_CALL= 0xc400_0199,
    }
}

pub const SUCCESS: usize = 0;

pub const VERSION: usize = (1 << 16) | 0;

extern crate alloc;

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, HOST_CALL, |_arg, ret, rmm, rec, run| {
        let rmi = rmm.rmi;
        let realmid = rec.rd.id();
        let vcpuid = rec.id();
        let ipa = rmi.get_reg(realmid, vcpuid, 1).unwrap_or(0x0);
        let pa: usize = ipa;
        unsafe {
            let host_call = HostCall::parse(pa);
            run.set_imm(host_call.imm());
            run.set_exit_reason(rmi::EXIT_HOST_CALL);

            trace!("HOST_CALL param: {:#X?}", host_call)
        };
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, ABI_VERSION, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let realmid = rec.rd.id();
        let vcpuid = rec.id();
        if rmi.set_reg(realmid, vcpuid, 0, VERSION).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        trace!("RSI_ABI_VERSION: {:#X?}", VERSION);
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, REALM_CONFIG, |_arg, ret, rmm, rec, _| {
        let rmi = rmm.rmi;
        let realmid = rec.rd.id();
        let vcpuid = rec.id();
        let config_ipa = rmi.get_reg(realmid, vcpuid, 1)?;
        rmi.realm_config(realmid, config_ipa)?;

        if rmi.set_reg(realmid, vcpuid, 0, SUCCESS).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, IPA_STATE_SET, |_arg, ret, rmm, rec, run| {
        let rmi = rmm.rmi;
        let realmid = rec.rd.id();
        let vcpuid = rec.id();
        let ipa_start = rmi.get_reg(realmid, vcpuid, 1)? as u64;
        let ipa_size = rmi.get_reg(realmid, vcpuid, 2)? as u64;
        let ipa_state = rmi.get_reg(realmid, vcpuid, 3)? as u8;
        // TODO: check ipa_state value, ipa address granularity
        unsafe {
            run.set_exit_reason(rmi::EXIT_RIPAS_CHANGE);
            run.set_ripas(ipa_start, ipa_size, ipa_state);
            rec.set_ripas(ipa_start, ipa_start + ipa_size, ipa_start, ipa_state);
            ret[0] = rmi::SUCCESS;
        };
        debug!(
            "RSI_IPA_STATE_SET: {:X} ~ {:X} {:X}",
            ipa_start,
            ipa_start + ipa_size,
            ipa_state
        );
        super::rmi::dummy();
        Ok(())
    });
}
