use crate::listen;
use crate::realm::context::{get_reg, set_reg};
use crate::rmi;
use crate::rsi::{RsiHandle, ERROR_INPUT, LOCAL_CHANNEL_SEND};

pub fn set_event_handler(rsi: &mut RsiHandle) {
    // Sends a client data size and local channel information to the host
    // and the host should deliver them to the server realm after checking a
    // exit reason 'EXIT_LOCAL_CHANNEL_SEND'
    listen!(rsi, LOCAL_CHANNEL_SEND, |_arg, ret, _rmm, rec, run| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;

        let lc_ipa = get_reg(realmid, vcpuid, 1)?;
        let lc_size = get_reg(realmid, vcpuid, 2)?;
        let data_size = get_reg(realmid, vcpuid, 3)?;

        trace!(
            "LOCAL_CHANNEL_SEND: lc_ipa: 0x{:X}, lc_size: 0x{:X}, data_size 0x{:X}",
            lc_ipa,
            lc_size,
            data_size
        );

        if lc_size < data_size {
            set_reg(realmid, vcpuid, 0, ERROR_INPUT)?;
            ret[0] = rmi::SUCCESS_REC_ENTER;
            error!(
                "LOCAL_CHANNEL_SEND failed due to ERROR_INPUT:
                lc_size: {:x}, data_size: {:x}",
                lc_size, data_size
            );
            return Ok(());
        }

        trace!("LOCAL_CHANNEL_SEND: Set exit_reason to EXIT_LOCAL_CHANNEL_SEND");
        unsafe {
            run.set_exit_reason(rmi::EXIT_LOCAL_CHANNEL_SEND);
            run.set_gpr(0, lc_ipa as u64)?;
            run.set_gpr(1, data_size as u64)?;
        }

        ret[0] = rmi::SUCCESS;
        Ok(())
    });
}
