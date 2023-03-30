use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_INIT_RIPAS, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    // related with:
    //   - RIM
    //   - REALM_MAP_MEMORY
    listen!(mainloop, rmi::DATA_CREATE, |ctx, rmi, _| {
        let data_addr = ctx.arg[0];
        let _rd_addr = ctx.arg[1];
        let _map_addr = ctx.arg[2];
        let src_addr = ctx.arg[3];

        let realm_id = 0; // temporarily
        let granule_sz = 4096;
        let ret = rmi.map(realm_id, data_addr, src_addr, granule_sz, 0);
        match ret {
            Ok(_) => ctx.ret[0] = rmi::SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    // related with:
    //   - REALM_MAP_MEMORY
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });
}
