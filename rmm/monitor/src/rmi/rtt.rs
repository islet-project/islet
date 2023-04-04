use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_INIT_RIPAS, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::DATA_CREATE, |ctx, rmi, _| {
        let _rd = ctx.arg[0];
        let _data = ctx.arg[1];
        let ipa = ctx.arg[2];
        let src = ctx.arg[3];

        let realm_id = 0; // temporarily
        let granule_sz = 4096;
        let ret = rmi.map(realm_id, ipa, src, granule_sz, 0);
        match ret {
            Ok(_) => ctx.ret[0] = rmi::SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    // Map from an Unprotected IPA to a Non-secure PA.
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |ctx, _, _| {
        let _rd = ctx.arg[0];
        let _ipa = ctx.arg[1];
        let _level = ctx.arg[2];
        let _src = ctx.arg[3];

        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });
}
