pub(crate) mod params;
pub(crate) mod rd;

pub use self::rd::Rd;

use self::params::Params;
use self::rd::State;
use super::error::Error;
use crate::event::Mainloop;
use crate::granule::{set_granule, GranuleState};
use crate::host;
use crate::listen;
use crate::measurement::HashContext;
use crate::mm::translation::PageTable;
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::IPATranslation;
use crate::realm::registry::RMS;
use crate::realm::vcpu::remove;
use crate::realm::Realm;
use crate::rmi;
use crate::{get_granule, get_granule_if};

use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::mutex::Mutex;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REALM_ACTIVATE, |arg, _, _| {
        let rd = arg[0];

        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let rd = rd_granule.content_mut::<Rd>();

        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }

        rd.set_state(State::Active);
        Ok(())
    });

    listen!(mainloop, rmi::REALM_CREATE, |arg, _, rmm| {
        let rd = arg[0];
        let params_ptr = arg[1];

        if rd == params_ptr {
            return Err(Error::RmiErrorInput);
        }

        let mut rd_granule = get_granule_if!(rd, GranuleState::Delegated)?;
        let rd_obj = rd_granule.content_mut::<Rd>();
        rmm.page_table.map(rd, true);

        let params = host::copy_from::<Params>(params_ptr).ok_or(Error::RmiErrorInput)?;
        params.verify_compliance(rd)?;

        let rtt_granule = get_granule_if!(params.rtt_base as usize, GranuleState::Delegated)?;
        // This is required to prevent from the deadlock in the below epilog
        // which acquires the same lock again
        core::mem::drop(rtt_granule);

        // revisit rmi.create_realm() (is it necessary?)
        create_realm(params.vmid, params.rtt_base as usize).map(|id| {
            rd_obj.init(
                id,
                params.rtt_base as usize,
                params.ipa_bits(),
                params.rtt_level_start as isize,
            )
        })?;

        let id = rd_obj.id();
        let rtt_base = rd_obj.rtt_base();
        // The below is added to avoid a fault regarding the RTT entry
        PageTable::get_ref().map(rtt_base, true);

        rd_obj.set_hash_algo(params.hash_algo);

        HashContext::new(rd_obj)?.measure_realm_create(&params)?;

        let mut eplilog = move || {
            let mut rtt_granule = get_granule_if!(rtt_base, GranuleState::Delegated)?;
            set_granule(&mut rtt_granule, GranuleState::RTT)?;
            set_granule(&mut rd_granule, GranuleState::RD)
        };

        eplilog().map_err(|e| {
            rmm.page_table.unmap(rd);
            remove(id).expect("Realm should be created before.");
            e
        })
    });

    listen!(mainloop, rmi::REC_AUX_COUNT, |_, ret, _| {
        ret[1] = rmi::MAX_REC_AUX_GRANULES;
        Ok(())
    });

    listen!(mainloop, rmi::REALM_DESTROY, |arg, _ret, rmm| {
        // get the lock for Rd
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        remove(rd.id())?;

        let mut rtt_granule = get_granule_if!(rd.rtt_base(), GranuleState::RTT)?;
        set_granule(&mut rtt_granule, GranuleState::Delegated)?;

        // change state when everything goes fine.
        set_granule(&mut rd_granule, GranuleState::Delegated)?;
        rmm.page_table.unmap(arg[0]);

        Ok(())
    });
}

fn create_realm(vmid: u16, rtt_base: usize) -> Result<usize, Error> {
    let mut rms = RMS.lock();

    for realm in rms.1.values() {
        if vmid == realm.lock().vmid {
            return Err(Error::RmiErrorInput);
        }
    }

    let id = rms.0;
    let s2_table = Arc::new(Mutex::new(
        Box::new(Stage2Translation::new(rtt_base)) as Box<dyn IPATranslation>
    ));
    let realm = Realm::new(id, vmid, s2_table);

    rms.0 += 1;
    rms.1.insert(id, realm.clone());

    Ok(id)
}
