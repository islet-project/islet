pub(crate) mod params;
pub(crate) mod rd;

use self::params::Params;
pub use self::rd::Rd;
use super::error::Error;
use crate::event::Mainloop;
use crate::host::pointer::Pointer as HostPointer;
use crate::listen;
use crate::rmi;
use crate::rmm::granule::{set_granule, GranuleState};
use crate::{get_granule, get_granule_if};

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REALM_ACTIVATE, |_, _, _| {
        super::dummy();
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
        rmm.mm.map(rd, true);

        let params = copy_from_host_or_ret!(Params, params_ptr, rmm.mm);
        if params.rtt_base as usize == rd {
            return Err(Error::RmiErrorInput);
        }

        // revisit rmi.create_realm() (is it necessary?)
        rmm.rmi
            .create_realm(params.vmid)
            .map(|id| rd_obj.init(id, params.rtt_base as usize))?;

        let id = rd_obj.id();
        let rtt_base = rd_obj.rtt_base();
        let mut eplilog = move || {
            let mut rtt_granule = get_granule_if!(rtt_base, GranuleState::Delegated)?;
            set_granule(&mut rtt_granule, GranuleState::RTT)?;
            set_granule(&mut rd_granule, GranuleState::RD)
        };

        eplilog().map_err(|e| {
            rmm.rmi.remove(id).expect("Realm should be created before.");
            e
        })
    });

    listen!(mainloop, rmi::REC_AUX_COUNT, |_, ret, _| {
        ret[1] = rmi::MAX_REC_AUX_GRANULES;
        Ok(())
    });

    listen!(mainloop, rmi::REALM_DESTROY, |arg, _ret, rmm| {
        let rmi = rmm.rmi;

        // get the lock for Rd
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        rmi.remove(rd.id())?;

        let mut rtt_granule = get_granule_if!(rd.rtt_base(), GranuleState::RTT)?;
        set_granule(&mut rtt_granule, GranuleState::Delegated)?;

        // change state when everything goes fine.
        set_granule(&mut rd_granule, GranuleState::Delegated)?;
        rmm.mm.unmap(arg[0]);

        Ok(())
    });
}
