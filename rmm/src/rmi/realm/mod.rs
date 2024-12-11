pub(crate) mod params;
pub(crate) use self::params::Params;
use super::error::Error;
use crate::event::RmiHandle;
use crate::granule::GRANULE_SIZE;
use crate::granule::{set_granule, GranuleState};
use crate::host;
use crate::listen;
use crate::measurement::{HashContext, MEASUREMENTS_SLOT_RIM};
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::IPATranslation;
use crate::realm::rd::State;
use crate::realm::rd::{insert_rtt, Rd};
use crate::realm::registry::{remove, VMID_SET};
use crate::rmi::{self, metadata::IsletRealmMetadata};
use crate::{get_granule, get_granule_if};

use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::mutex::Mutex;

extern crate alloc;

pub fn set_event_handler(rmi: &mut RmiHandle) {
    #[cfg(any(not(kani), feature = "mc_rmi_realm_activate"))]
    listen!(rmi, rmi::REALM_ACTIVATE, |arg, _, _| {
        let rd = arg[0];
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;

        if let Some(meta) = rd.metadata() {
            debug!("Realm metadata is in use!");
            let g_metadata = get_granule_if!(meta, GranuleState::Metadata)?;
            let metadata = g_metadata.content::<IsletRealmMetadata>()?;

            if !metadata.equal_rd_rim(&rd.measurements[MEASUREMENTS_SLOT_RIM]) {
                error!("Calculated rim and those read from metadata are not the same!");
                return Err(Error::RmiErrorRealm(0));
            }

            if !metadata.equal_rd_hash_algo(rd.hash_algo()) {
                error!("Provided measurement hash algorithm and metadata hash algorithm are different!");
                return Err(Error::RmiErrorRealm(0));
            }
        }

        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }

        rd.set_state(State::Active);
        Ok(())
    });

    #[cfg(not(kani))]
    listen!(rmi, rmi::REALM_CREATE, |arg, _, rmm| {
        let rd = arg[0];
        let params_ptr = arg[1];

        if rd == params_ptr {
            return Err(Error::RmiErrorInput);
        }

        let mut rd_granule = get_granule_if!(rd, GranuleState::Delegated)?;
        let mut rd_obj = rd_granule.content_mut::<Rd>()?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(rd, true);

        rmm.page_table.map(params_ptr, false);
        let params = host::copy_from::<Params>(params_ptr).ok_or(Error::RmiErrorInput)?;
        rmm.page_table.unmap(params_ptr);
        params.verify_compliance(rd)?;

        for i in 0..params.rtt_num_start as usize {
            let rtt = params.rtt_base as usize + i * GRANULE_SIZE;
            let _ = get_granule_if!(rtt, GranuleState::Delegated)?;
            // The below is added to avoid a fault regarding the RTT entry
            // during the below stage 2 page table creation
            rmm.page_table.map(rtt, true);
        }

        // revisit rmi.create_realm() (is it necessary?)
        create_realm(params.vmid as usize).map(|_| {
            let s2 = Box::new(Stage2Translation::new(
                params.rtt_base as usize,
                params.rtt_level_start as usize,
                params.rtt_num_start as usize,
            )) as Box<dyn IPATranslation>;

            insert_rtt(params.vmid as usize, Arc::new(Mutex::new(s2)));

            rd_obj.init(
                params.vmid,
                params.rtt_base as usize,
                params.rtt_num_start as usize,
                params.ipa_bits(),
                params.rtt_level_start as isize,
                params.rpv,
                params.sve_en(),
                params.sve_vl as u64,
            )
        })?;

        let rtt_base = rd_obj.rtt_base();
        rd_obj.set_hash_algo(params.hash_algo);

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(&mut rd_obj)?.measure_realm_create(&params)?;

        let mut epilogue = move || {
            for i in 0..params.rtt_num_start as usize {
                let rtt = rtt_base + i * GRANULE_SIZE;
                let mut rtt_granule = get_granule_if!(rtt, GranuleState::Delegated)?;
                set_granule(&mut rtt_granule, GranuleState::RTT)?;
            }
            set_granule(&mut rd_granule, GranuleState::RD)
        };

        epilogue().map_err(|e| {
            #[cfg(not(kani))]
            // `page_table` is currently not reachable in model checking harnesses
            rmm.page_table.unmap(rd);
            remove(params.vmid as usize).expect("Realm should be created before.");
            e
        })
    });

    #[cfg(any(not(kani), feature = "mc_rmi_rec_aux_count"))]
    listen!(rmi, rmi::REC_AUX_COUNT, |arg, ret, _| {
        let _ = get_granule_if!(arg[0], GranuleState::RD)?;
        ret[1] = rmi::MAX_REC_AUX_GRANULES;
        Ok(())
    });

    #[cfg(any(not(kani), feature = "mc_rmi_realm_destroy"))]
    listen!(rmi, rmi::REALM_DESTROY, |arg, _ret, rmm| {
        // get the lock for Rd
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        if rd_granule.num_children() > 0 {
            return Err(Error::RmiErrorRealm(0));
        }
        let mut rd = rd_granule.content::<Rd>()?;
        let vmid = rd.id();
        let rtt_base = rd.rtt_base();

        if let Some(meta) = rd.metadata() {
            let mut meta_granule = get_granule_if!(meta, GranuleState::Metadata)?;
            set_granule(&mut meta_granule, GranuleState::Delegated)?;
            rd.set_metadata(None);
        }

        #[cfg(kani)]
        // XXX: the below can be guaranteed by Rd's invariants
        kani::assume(crate::granule::validate_addr(rtt_base));

        #[cfg(not(feature = "gst_page_table"))]
        {
            #[cfg(not(kani))]
            for i in 0..rd.rtt_num_start() {
                let rtt = rtt_base + i * GRANULE_SIZE;

                let rtt_granule = get_granule!(rtt)?;
                if rtt_granule.num_children() > 0 {
                    return Err(Error::RmiErrorRealm(0));
                }
            }
            #[cfg(kani)]
            {
                // XXX: we remove the loop and consider only the first iteration
                //      to reduce the overall verification time
                let rtt_granule = get_granule!(rtt_base)?;
                if rtt_granule.num_children() > 0 {
                    return Err(Error::RmiErrorRealm(0));
                }
            }
        }

        #[cfg(not(kani))]
        for i in 0..rd.rtt_num_start() {
            let rtt = rtt_base + i * GRANULE_SIZE;
            let mut rtt_granule = get_granule!(rtt)?;
            set_granule(&mut rtt_granule, GranuleState::Delegated)?;
        }
        #[cfg(kani)]
        {
            // XXX: we remove the loop and consider only the first iteration
            //      to reduce the overall verification time
            let mut rtt_granule = get_granule!(rtt_base)?;
            set_granule(&mut rtt_granule, GranuleState::Delegated)?;
        }

        // change state when everything goes fine.
        set_granule(&mut rd_granule, GranuleState::Delegated)?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.unmap(arg[0]);
        // TODO: remove the below after modeling `VmidIsFree()`
        #[cfg(not(kani))]
        remove(vmid)?;

        Ok(())
    });

    // ISLET_REALM_SET_METADATA is a vendor specific RMI for provisioning the realm metadata to the Realm
    // Input registers
    // x0: function id (0xC7000150)
    // x1: rd - a physicall address of the RD for the target Realm
    // x2: mdg - a physicall address of the delegated granule used for storage of the metadata
    // x3: meta_ptr - a physicall address of the host provided (NS) metadata granule
    listen!(rmi, rmi::ISLET_REALM_SET_METADATA, |arg, _ret, _rmm| {
        let rd_addr = arg[0];
        let mdg_addr = arg[1];
        let meta_ptr = arg[2];

        let realm_metadata = Box::new(IsletRealmMetadata::from_ns(meta_ptr)?);
        realm_metadata.dump();

        if let Err(e) = realm_metadata.verify_signature() {
            error!("Verification of realm metadata signature has failed");
            Err(e)?;
        }

        if let Err(e) = realm_metadata.validate() {
            error!("The content of realm metadata is not valid");
            Err(e)?;
        }

        let mut rd_granule = get_granule_if!(rd_addr, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;

        if !rd.at_state(State::New) {
            error!("Metadata can only be set for new realms");
            Err(Error::RmiErrorRealm(0))?;
        }

        if rd.metadata().is_some() {
            error!("Metadata is already set");
            Err(Error::RmiErrorRealm(0))?;
        }

        let mut g_metadata = get_granule_if!(mdg_addr, GranuleState::Delegated)?;
        let mut meta = g_metadata.content_mut::<IsletRealmMetadata>()?;
        *meta = *realm_metadata.clone();

        set_granule(&mut g_metadata, GranuleState::Metadata)?;

        rd.set_metadata(Some(mdg_addr));

        Ok(())
    });
}

fn create_realm(vmid: usize) -> Result<(), Error> {
    let mut vmid_set = VMID_SET.lock();
    if vmid_set.contains(&vmid) {
        return Err(Error::RmiErrorInput);
    } else {
        vmid_set.insert(vmid);
    };

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::realm::rd::{Rd, State};
    use crate::rmi::{ERROR_INPUT, REALM_ACTIVATE, REALM_CREATE, SUCCESS};
    use crate::test_utils::*;

    use alloc::vec;

    #[test]
    fn rmi_realm_create_positive() {
        let rd = realm_create();

        let ret = rmi::<REALM_ACTIVATE>(&[rd]);
        assert_eq!(ret[0], SUCCESS);

        unsafe {
            let rd_obj = &*(rd as *const Rd);
            assert!(rd_obj.at_state(State::Active));
        };

        realm_destroy(rd);

        miri_teardown();
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_realm_create
    /*
        Check 1 : params_align; rd : 0x88300000 params_ptr : 0x88303001 ret : 1
        Check 2 : params_bound; rd : 0x88300000 params_ptr : 0x1C0B0000 ret : 1
        Check 3 : params_bound; rd : 0x88300000 params_ptr : 0x1000000001000 ret : 1
        Check 4 : params_pas; rd : 0x88300000 params_ptr : 0x88309000 ret : 1
        Check 5 : params_pas; rd : 0x88300000 params_ptr : 0x6000000 ret : 1
        Check 6 : params_valid; rd : 0x88300000 params_ptr : 0x8830C000 ret : 1
        Check 7 : params_valid; rd : 0x88300000 params_ptr : 0x8830F000 ret : 1
        Check 8 : params_supp
        Skipping Test case
        Check 9 : params_supp; rd : 0x88300000 params_ptr : 0x88315000 ret : 1
        Check 10 : params_supp; rd : 0x88300000 params_ptr : 0x88318000 ret : 1
        Check 11 : params_supp; rd : 0x88300000 params_ptr : 0x8831B000 ret : 1
        Check 12 : params_supp; rd : 0x88300000 params_ptr : 0x8831E000 ret : 1
        Check 13 : params_supp; rd : 0x88300000 params_ptr : 0x88321000 ret : 1
        Check 14 : alias; rd : 0x88306000 params_ptr : 0x88303000 ret : 1
        Check 15 : rd_align; rd : 0x88300001 params_ptr : 0x88303000 ret : 1
        Check 16 : rd_bound; rd : 0x1C0B0000 params_ptr : 0x88303000 ret : 1
        Check 17 : rd_bound; rd : 0x1000000001000 params_ptr : 0x88303000 ret : 1
        Check 18 : rd_state; rd : 0x88324000 params_ptr : 0x88303000 ret : 1
        Check 19 : rd_state; rd : 0x88327000 params_ptr : 0x88303000 ret : 1
        Check 20 : rd_state; rd : 0x88336000 params_ptr : 0x88303000 ret : 1
        Check 21 : rd_state; rd : 0x8832A000 params_ptr : 0x88303000 ret : 1
        Check 22 : rd_state; rd : 0x88372000 params_ptr : 0x88303000 ret : 1
        Check 23 : rtt_align; rd : 0x88300000 params_ptr : 0x88378000 ret : 1
        Check 24 : rtt_num_level; rd : 0x88300000 params_ptr : 0x8837B000 ret : 1
        Check 25 : rtt_state; rd : 0x88300000 params_ptr : 0x8837E000 ret : 1
        Check 26 : vmid_valid
        Couldn't create VMID Invalid sequence
        Skipping Test case
        Check 27 : vmid_valid; rd : 0x88300000 params_ptr : 0x88387000 ret : 1
    */
    #[test]
    fn rmi_realm_create_negative() {
        let test_data = vec![
            // TODO: Cover all test data
            ((0x88300000 as usize, 0x88303001 as usize), ERROR_INPUT),
        ];

        // main test
        for (input, output) in test_data {
            let ret = rmi::<REALM_CREATE>(&[input.0, input.1, 0]);
            assert_eq!(output, ret[0]);
        }
    }
}
