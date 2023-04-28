pub(crate) mod params;
pub(crate) mod rd;

use self::params::Params;
pub use self::rd::Rd;
use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmm::granule;
use crate::rmm::granule::{GranuleState, RmmGranule};

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REALM_ACTIVATE, |ctx, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REALM_CREATE, |ctx, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        let params_ptr = ctx.arg[1];

        let g_rd = granule::find_granule(ctx.arg[0], GranuleState::Delegated).unwrap();
        let g_param = granule::find_granule(params_ptr, GranuleState::Undelegated).unwrap();
        g_rd.set_state(GranuleState::RD, mm);
        g_param.set_state(GranuleState::Param, mm);

        let param = unsafe { Params::parse(params_ptr) };
        trace!("{:?}", param);

        g_param.set_state(GranuleState::Undelegated, mm);

        // TODO:
        //   Validate params
        //   Manage granule including locking
        //   Manage vmid
        //   Keep params in Realm

        let ret = rmi.create_realm();
        match ret {
            Ok(id) => {
                ctx.ret[0] = rmi::SUCCESS;
                let _ = unsafe { Rd::new(ctx.arg[0], id) };
                ctx.ret[1] = id;
            }
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    listen!(mainloop, rmi::REC_AUX_COUNT, |ctx, _| {
        ctx.ret[0] = rmi::SUCCESS;
        ctx.ret[1] = rmi::MAX_REC_AUX_GRANULES;
    });

    listen!(mainloop, rmi::REALM_DESTROY, |ctx, rmm| {
        let rmi = rmm.rmi;
        let _rd = unsafe { Rd::into(ctx.arg[0]) };
        let ret = rmi.remove(0); // temporarily

        let g_rd = granule::find_granule(ctx.arg[0], GranuleState::RD).unwrap();
        g_rd.set_state(GranuleState::Delegated, rmm.mm);

        match ret {
            Ok(_) => ctx.ret[0] = rmi::SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });
}
