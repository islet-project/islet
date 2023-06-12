pub mod constraint;
pub mod hostcall;

use crate::event::RsiHandle;
use crate::listen;
use crate::rmi;
use crate::rsi::hostcall::HostCall;

pub const IPA_STATE_SET: usize = 0xc400_0197;
pub const HOST_CALL: usize = 0xc400_0199;

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
}
