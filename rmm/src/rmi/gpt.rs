use crate::asm::{smc, SMC_SUCCESS};
use crate::event::Mainloop;
use crate::granule::{set_granule, GranuleState};
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
#[cfg(not(feature = "gst_page_table"))]
use crate::{get_granule, get_granule_if};
#[cfg(feature = "gst_page_table")]
use crate::{get_granule, get_granule_if, set_state_and_get_granule};

#[cfg(feature = "gst_page_table")]
use vmsa::error::Error as MmError;

extern crate alloc;

// defined in trusted-firmware-a/include/services/rmmd_svc.h
const MARK_REALM: usize = 0xc400_01b0;
const MARK_NONSECURE: usize = 0xc400_01b1;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::GRANULE_DELEGATE, |arg, _, rmm| {
        let addr = arg[0];
        #[cfg(feature = "gst_page_table")]
        let mut granule = match get_granule_if!(addr, GranuleState::Undelegated) {
            Err(MmError::MmNoEntry) => set_state_and_get_granule!(addr, GranuleState::Undelegated),
            other => other,
        }?;
        #[cfg(not(feature = "gst_page_table"))]
        let mut granule = get_granule_if!(addr, GranuleState::Undelegated)?;

        if smc(MARK_REALM, &[addr])[0] != SMC_SUCCESS {
            return Err(Error::RmiErrorInput);
        }

        rmm.page_table.map(addr, true);
        set_granule(&mut granule, GranuleState::Delegated).map_err(|e| {
            rmm.page_table.unmap(addr);
            e
        })?;
        rmm.page_table.unmap(addr);
        Ok(())
    });

    listen!(mainloop, rmi::GRANULE_UNDELEGATE, |arg, _, rmm| {
        let addr = arg[0];
        let mut granule = get_granule_if!(addr, GranuleState::Delegated)?;

        if smc(MARK_NONSECURE, &[addr])[0] != SMC_SUCCESS {
            panic!(
                "A delegated granule should only be undelegated on request from RMM. {:X}",
                addr
            );
        }

        rmm.page_table.map(addr, false);
        set_granule(&mut granule, GranuleState::Undelegated).map_err(|e| {
            rmm.page_table.unmap(addr);
            e
        })?;
        rmm.page_table.unmap(addr);
        Ok(())
    });
}
