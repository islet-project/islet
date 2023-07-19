pub(crate) mod params;
pub(crate) mod rd;

use self::params::Params;
pub use self::rd::Rd;
use crate::event::Mainloop;
use crate::host::pointer::Pointer as HostPointer;
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmm::granule;
use crate::rmm::granule::GranuleState;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REALM_ACTIVATE, |_, _, _| {
        super::dummy();
        Ok(())
    });

    listen!(mainloop, rmi::REALM_CREATE, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        let params = copy_from_host_or_ret!(Params, arg[1], mm);
        trace!("{:?}", params);

        if granule::set_granule(arg[0], GranuleState::RD, mm) != granule::RET_SUCCESS {
            return Err(Error::RmiErrorInput);
        }

        // TODO:
        //   Validate params
        //   Manage granule including locking
        //   Manage vmid
        //   Keep params in Realm

        let res = rmi.create_realm();
        match res {
            Ok(id) => {
                let _ = unsafe { Rd::new(arg[0], id) };
                ret[1] = id;
            }
            Err(val) => return Err(val),
        }
        Ok(())
    });

    listen!(mainloop, rmi::REC_AUX_COUNT, |_, ret, _| {
        ret[1] = rmi::MAX_REC_AUX_GRANULES;
        Ok(())
    });

    listen!(mainloop, rmi::REALM_DESTROY, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let _rd = unsafe { Rd::into(arg[0]) };
        let res = rmi.remove(0); // temporarily
        if granule::set_granule(arg[0], GranuleState::Delegated, rmm.mm) != granule::RET_SUCCESS {
            return Err(Error::RmiErrorInput);
        }

        match res {
            Ok(_) => {}
            Err(val) => return Err(val),
        }
        Ok(())
    });
}
