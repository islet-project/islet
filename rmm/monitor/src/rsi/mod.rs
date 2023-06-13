pub mod constraint;
pub mod hostcall;
pub mod psci;

use crate::event::RsiHandle;
use crate::listen;
use crate::rmi;
use crate::rsi::hostcall::HostCall;

pub const ABI_VERSION: usize = 0xc400_0190;
pub const MEASUREMENT_READ: usize = 0xc400_0192;
pub const MEASUREMENT_EXTEND: usize = 0xc400_0193;
pub const ATTEST_TOKEN_INIT: usize = 0xc400_0194;
pub const ATTEST_TOKEN_CONTINUE: usize = 0xc400_0195;
pub const REALM_CONFIG: usize = 0xc400_0196;
pub const IPA_STATE_SET: usize = 0xc400_0197;
pub const HOST_CALL: usize = 0xc400_0199;

pub const RSI_SUCCESS: usize = 0;

pub const VERSION: usize = (1 << 16) | 0;

extern crate alloc;

pub fn set_event_handler(rsi: &mut RsiHandle) {
    listen!(rsi, HOST_CALL, |arg, ret, rmm, run| {
        let rmi = rmm.rmi;
        let realmid = arg[0];
        let vcpuid = arg[1];
        let ipa = rmi.get_reg(realmid, vcpuid, 1).unwrap_or(0x0);
        let pa: usize = ipa;
        unsafe {
            let host_call = HostCall::parse(pa);
            run.set_imm(host_call.imm());
            run.set_exit_reason(rmi::EXIT_HOST_CALL);

            trace!("HOST_CALL param: {:#X?}", host_call)
        };
        ret[0] = rmi::SUCCESS;
    });

    listen!(rsi, ABI_VERSION, |arg, ret, rmm, _| {
        let rmi = rmm.rmi;
        let realmid = arg[0];
        let vcpuid = arg[1];
        if rmi.set_reg(realmid, vcpuid, 0, VERSION).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        trace!("RSI_ABI_VERSION: {:#X?}", VERSION);
        ret[0] = rmi::SUCCESS_REC_ENTER;
    });

    listen!(rsi, REALM_CONFIG, |arg, ret, rmm, _| {
        let rmi = rmm.rmi;
        let realmid = arg[0];
        let vcpuid = arg[1];
        let _config_ipa = rmi.get_reg(realmid, vcpuid, 0);
        if rmi.set_reg(realmid, vcpuid, 0, RSI_SUCCESS).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        super::rmi::dummy();
        ret[0] = rmi::SUCCESS_REC_ENTER;
    });

    listen!(rsi, IPA_STATE_SET, |arg, ret, rmm, run| {
        let rmi = rmm.rmi;
        let realmid = arg[0];
        let vcpuid = arg[1];
        let ipa_start = rmi.get_reg(realmid, vcpuid, 1).unwrap_or(0x0) as u64;
        let ipa_size = rmi.get_reg(realmid, vcpuid, 2).unwrap_or(0x0) as u64;
        let ipa_state = rmi.get_reg(realmid, vcpuid, 3).unwrap_or(0x0) as u8;
        // TODO: check ipa_state value, ipa address granularity
        // TODO: save this into rec;
        unsafe {
            run.set_exit_reason(rmi::EXIT_RIPAS_CHANGE);
            run.set_ripas(ipa_start, ipa_size, ipa_state);
            ret[0] = rmi::SUCCESS;
        };
        debug!(
            "RSI_IPA_STATE_SET: {:X} ~ {:X} {:X}",
            ipa_start,
            ipa_start + ipa_size,
            ipa_state
        );
        super::rmi::dummy();
    });
}
