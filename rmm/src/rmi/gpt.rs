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
pub const MARK_REALM: usize = 0xc400_01b0;
pub const MARK_NONSECURE: usize = 0xc400_01b1;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    #[cfg(any(not(kani), feature = "mc_rmi_granule_delegate"))]
    listen!(mainloop, rmi::GRANULE_DELEGATE, |arg, _, rmm| {
        let addr = arg[0];

        #[cfg(feature = "gst_page_table")]
        let mut granule = match get_granule_if!(addr, GranuleState::Undelegated) {
            Err(MmError::MmNoEntry) => set_state_and_get_granule!(addr, GranuleState::Undelegated),
            other => other,
        }?;
        #[cfg(not(feature = "gst_page_table"))]
        let mut granule = get_granule_if!(addr, GranuleState::Undelegated)?;

        // Avoid deadlock in get_granule() in smc() on {miri, test} mode
        #[cfg(any(miri, test))]
        core::mem::drop(granule);

        if smc(MARK_REALM, &[addr])[0] != SMC_SUCCESS {
            return Err(Error::RmiErrorInput);
        }

        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(addr, true);

        #[cfg(any(miri, test))]
        let mut granule = get_granule_if!(addr, GranuleState::Undelegated)?;
        set_granule(&mut granule, GranuleState::Delegated).map_err(|e| {
            #[cfg(not(kani))]
            // `page_table` is currently not reachable in model checking harnesses
            rmm.page_table.unmap(addr);
            e
        })?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.unmap(addr);
        Ok(())
    });

    #[cfg(any(not(kani), feature = "mc_rmi_granule_undelegate"))]
    listen!(mainloop, rmi::GRANULE_UNDELEGATE, |arg, _, rmm| {
        let addr = arg[0];
        let mut granule = get_granule_if!(addr, GranuleState::Delegated)?;

        // Avoid deadlock in get_granule() in smc() on {miri, test} mode
        #[cfg(any(miri, test))]
        core::mem::drop(granule);

        if smc(MARK_NONSECURE, &[addr])[0] != SMC_SUCCESS {
            panic!(
                "A delegated granule should only be undelegated on request from RMM. {:X}",
                addr
            );
        }

        #[cfg(any(miri, test))]
        let mut granule = get_granule_if!(addr, GranuleState::Delegated)?;

        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(addr, false);
        set_granule(&mut granule, GranuleState::Undelegated).map_err(|e| {
            #[cfg(not(kani))]
            // `page_table` is currently not reachable in model checking harnesses
            rmm.page_table.unmap(addr);
            e
        })?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.unmap(addr);
        Ok(())
    });
}

#[cfg(test)]
mod test {
    use crate::rmi::gpt::GranuleState;
    use crate::rmi::{ERROR_INPUT, GRANULE_DELEGATE, GRANULE_UNDELEGATE, SUCCESS};
    use crate::test_utils::*;
    use crate::{get_granule, get_granule_if};

    use alloc::vec;

    #[test]
    fn rmi_granule_delegate_positive() {
        let mocking_addr = mock::host::alloc_granule(IDX_RD);
        let ret = rmi::<GRANULE_DELEGATE>(&[mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
        assert!(get_granule_if!(mocking_addr, GranuleState::Delegated).is_ok());

        let ret = rmi::<GRANULE_UNDELEGATE>(&[mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
        assert!(get_granule_if!(mocking_addr, GranuleState::Undelegated).is_ok());

        miri_teardown();
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_granule_delegate
    /*
       Check 1 : gran_align; intent id : 0x0 addr : 0x88300001
       Check 2 : gran_bound; intent id : 0x1 addr : 0x1C0B0000
       Check 3 : gran_bound; intent id : 0x2 addr : 0x1000000001000
       Check 4 : gran_state; intent id : 0x3 addr : 0x8830C000
       Check 5 : gran_state; intent id : 0x4 addr : 0x88315000
       Check 6 : gran_state; intent id : 0x5 addr : 0x88351000
       Check 7 : gran_state; intent id : 0x6 addr : 0x88306000
       Check 8 : gran_state; intent id : 0x7 addr : 0x88303000
       Check 9 : gran_gpt; intent id : 0x8 addr : 0x88357000
       Check 10 : gran_gpt; intent id : 0x9 addr : 0x6000000
    */
    #[test]
    fn rmi_granule_delegate_negative() {
        let test_data = vec![
            (0x88300001, ERROR_INPUT),
            (0x1C0B0000, ERROR_INPUT),
            (0x1000000001000, ERROR_INPUT),
            // TODO: Cover all test data
        ];

        for (input, output) in test_data {
            let ret = rmi::<GRANULE_DELEGATE>(&[input]);
            assert_eq!(output, ret[0]);
        }
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_granule_undelegate
    /*
       Check 1 : gran_align; intent id : 0x0 addr : 0x88300001
       Check 2 : gran_bound; intent id : 0x1 addr : 0x1C0B0000
       Check 3 : gran_bound; intent id : 0x2 addr : 0x1000000001000
       Check 4 : gran_state; intent id : 0x3 addr : 0x8830C000
       Check 5 : gran_state; intent id : 0x4 addr : 0x88315000
       Check 6 : gran_state; intent id : 0x5 addr : 0x88351000
       Check 7 : gran_state; intent id : 0x6 addr : 0x88306000
       Check 8 : gran_state; intent id : 0x7 addr : 0x88303000
    */
    #[test]
    fn rmi_granule_undelegate() {
        let test_data = vec![
            (0x88300001, ERROR_INPUT),
            (0x1C0B0000, ERROR_INPUT),
            (0x1000000001000, ERROR_INPUT),
            (0x8830C000, ERROR_INPUT),
            // TODO: Cover all test data
        ];

        for (input, output) in test_data {
            let ret = rmi::<GRANULE_UNDELEGATE>(&[input]);
            assert_eq!(output, ret[0]);
        }

        miri_teardown();
    }
}
