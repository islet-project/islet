pub(crate) mod params;
pub(crate) mod rd;

use self::params::Params;
pub use self::rd::Rd;
use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmm::granule;
use crate::rmm::granule::GranuleState;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REALM_ACTIVATE, |_, ret, _| {
        super::dummy();
        ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REALM_CREATE, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        let params_ptr = arg[1];
        mm.map(params_ptr, false);
        if granule::set_granule(arg[0], GranuleState::RD, mm) != granule::RET_SUCCESS {
            ret[0] = rmi::ERROR_INPUT;
            return;
        }

        let param = unsafe { Params::parse(params_ptr) };
        trace!("{:?}", param);

        // TODO:
        //   Validate params
        //   Manage granule including locking
        //   Manage vmid
        //   Keep params in Realm

        let res = rmi.create_realm();
        match res {
            Ok(id) => {
                ret[0] = rmi::SUCCESS;
                let _ = unsafe { Rd::new(arg[0], id) };
                ret[1] = id;
            }
            Err(_) => ret[0] = rmi::RET_FAIL,
        }
        mm.unmap(params_ptr);
    });

    listen!(mainloop, rmi::REC_AUX_COUNT, |_, ret, _| {
        ret[0] = rmi::SUCCESS;
        ret[1] = rmi::MAX_REC_AUX_GRANULES;
    });

    listen!(mainloop, rmi::REALM_DESTROY, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let _rd = unsafe { Rd::into(arg[0]) };
        let res = rmi.remove(0); // temporarily
        if granule::set_granule(arg[0], GranuleState::Delegated, rmm.mm) != granule::RET_SUCCESS {
            ret[0] = rmi::ERROR_INPUT;
            return;
        }

        match res {
            Ok(_) => ret[0] = rmi::SUCCESS,
            Err(_) => ret[0] = rmi::RET_FAIL,
        }
    });
}
