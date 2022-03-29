use monitor::eprintln;
use monitor::io::Write;
use monitor::mainloop::Mainloop;

use armv9a::smc;

use crate::rmi;

pub fn set_event_handler(mainloop: &mut Mainloop<rmi::Receiver>) {
    mainloop.set_event_handler(rmi::Code::GranuleDelegate, |call| {
        let cmd = usize::from(smc::Code::MarkRealm);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        call.reply(ret[0])
            .err()
            .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });

    mainloop.set_event_handler(rmi::Code::GranuleUndelegate, |call| {
        let cmd = usize::from(smc::Code::MarkNonSecure);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        call.reply(ret[0])
            .err()
            .map(|e| eprintln!("RMM: failed to reply - {:?}", e));
    });
}
