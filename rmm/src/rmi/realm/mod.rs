pub(crate) mod params;
use self::params::Params;
use super::error::Error;
use crate::event::Mainloop;
use crate::granule::GRANULE_SIZE;
use crate::granule::{set_granule, GranuleState};
use crate::host;
use crate::listen;
use crate::measurement::HashContext;
use crate::mm::translation::PageTable;
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::IPATranslation;
use crate::realm::rd::State;
use crate::realm::rd::{insert_rtt, Rd};
use crate::realm::registry::{remove, VMID_SET};
use crate::rmi;
use crate::{get_granule, get_granule_if};

use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::mutex::Mutex;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    #[cfg(any(not(kani), feature = "mc_rmi_realm_activate"))]
    listen!(mainloop, rmi::REALM_ACTIVATE, |arg, _, _| {
        let rd = arg[0];

        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;

        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }

        rd.set_state(State::Active);
        Ok(())
    });

    #[cfg(not(kani))]
    listen!(mainloop, rmi::REALM_CREATE, |arg, _, rmm| {
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

        let params = host::copy_from::<Params>(params_ptr).ok_or(Error::RmiErrorInput)?;
        params.verify_compliance(rd)?;

        for i in 0..params.rtt_num_start as usize {
            let rtt = params.rtt_base as usize + i * GRANULE_SIZE;
            let _ = get_granule_if!(rtt, GranuleState::Delegated)?;
            // The below is added to avoid a fault regarding the RTT entry
            // during the below stage 2 page table creation
            PageTable::get_ref().map(rtt, true);
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
            )
        })?;

        let rtt_base = rd_obj.rtt_base();
        rd_obj.set_hash_algo(params.hash_algo);

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(&mut rd_obj)?.measure_realm_create(&params)?;

        let mut eplilog = move || {
            for i in 0..params.rtt_num_start as usize {
                let rtt = rtt_base + i * GRANULE_SIZE;
                let mut rtt_granule = get_granule_if!(rtt, GranuleState::Delegated)?;
                set_granule(&mut rtt_granule, GranuleState::RTT)?;
            }
            set_granule(&mut rd_granule, GranuleState::RD)
        };

        eplilog().map_err(|e| {
            #[cfg(not(kani))]
            // `page_table` is currently not reachable in model checking harnesses
            rmm.page_table.unmap(rd);
            remove(params.vmid as usize).expect("Realm should be created before.");
            e
        })
    });

    #[cfg(any(not(kani), feature = "mc_rmi_rec_aux_count"))]
    listen!(mainloop, rmi::REC_AUX_COUNT, |arg, ret, _| {
        let _ = get_granule_if!(arg[0], GranuleState::RD)?;
        ret[1] = rmi::MAX_REC_AUX_GRANULES;
        Ok(())
    });

    #[cfg(not(kani))]
    listen!(mainloop, rmi::REALM_DESTROY, |arg, _ret, rmm| {
        // get the lock for Rd
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        #[cfg(feature = "gst_page_table")]
        if rd_granule.num_children() > 0 {
            return Err(Error::RmiErrorRealm(0));
        }
        let rd = rd_granule.content::<Rd>()?;
        let vmid = rd.id();

        #[cfg(feature = "gst_page_table")]
        if rd_granule.num_children() > 0 {
            return Err(Error::RmiErrorRealm(0));
        }

        let rtt_base = rd.rtt_base();
        for i in 0..rd.rtt_num_start() {
            let rtt = rtt_base + i * GRANULE_SIZE;
            let mut rtt_granule = get_granule_if!(rtt, GranuleState::RTT)?;
            set_granule(&mut rtt_granule, GranuleState::Delegated)?;
        }

        // change state when everything goes fine.
        set_granule(&mut rd_granule, GranuleState::Delegated)?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.unmap(arg[0]);
        remove(vmid)?;

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
