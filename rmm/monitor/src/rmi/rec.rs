use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    // related with:
    //   - VCPU_CREATE
    listen!(mainloop, rmi::REC_CREATE, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    // related with:
    //   - REALM_RUN
    listen!(mainloop, rmi::REC_ENTER, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::ERROR_REC;
    });
}
