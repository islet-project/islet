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

    listen!(mainloop, rmi::REALM_CREATE, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;

        if arg[0] == arg[1] {
            return Err(Error::RmiErrorInput);
        }

        // get the lock for granule.
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::Delegated)?;
        let rd = rd_granule.content_mut::<Rd>();
        mm.map(arg[0], true);

        // read params
        let params = copy_from_host_or_ret!(Params, arg[1], mm);
        trace!("{:?}", params);
        params.validate()?;

        // TODO:
        //   Manage vmid
        //   Keep params in Realm
        //   revisit rmi.create_realm() (is it necessary?)

        rmi.create_realm().map(|id| {
            rd.init(id, params.rtt_base as usize);
            ret[1] = id;
        })?;

        if arg[0] == rd.rtt_base() {
            return Err(Error::RmiErrorInput);
        }
        let mut rtt_granule = get_granule_if!(rd.rtt_base(), GranuleState::Delegated)?;
        set_granule(&mut rtt_granule, GranuleState::RTT)?;

        // set Rd state only when everything goes well.
        set_granule(&mut rd_granule, GranuleState::RD)?;

        Ok(())
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
