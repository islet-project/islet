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
    listen!(mainloop, rmi::DATA_CREATE, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    // related with:
    //   - REALM_MAP_MEMORY
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });
}
