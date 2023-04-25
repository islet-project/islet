pub(crate) mod params;
pub(crate) mod rd;

use self::params::Params;
pub use self::rd::Rd;
use super::gpt::{mark_ns, mark_realm};
use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REALM_ACTIVATE, |ctx, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::REALM_CREATE, |ctx, rmm| {
        let rmi = rmm.rmi;
        let smc = rmm.smc;
        let mm = rmm.mm;
        let _ = mm.map([ctx.arg[0], ctx.arg[1], 0, 0]);
        let rd = unsafe { &mut Rd::new(ctx.arg[0]) };
        let params_ptr = ctx.arg[1];

        // TODO: Read ns memory w/o delegation
        if mark_realm(smc, params_ptr)[0] == 0 {
            let param = unsafe { Params::parse(params_ptr) };
            trace!("{:?}", param);
            let _ = mark_ns(smc, params_ptr)[0];
        } else {
            ctx.ret[0] = rmi::RET_FAIL;
            return;
        }

        // TODO:
        //   Validate params
        //   Manage granule including locking
        //   Manage vmid
        //   Keep params in Realm

        let ret = rmi.create_realm();
        match ret {
            Ok(id) => {
                ctx.ret[0] = rmi::SUCCESS;
                rd.realm_id = id;
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
        match ret {
            Ok(_) => ctx.ret[0] = rmi::SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });
}
