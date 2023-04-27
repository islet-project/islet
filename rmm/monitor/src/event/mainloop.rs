extern crate alloc;

use super::Context;
use crate::rmi;
use crate::smc::SecureMonitorCall;
use crate::utils::spsc;
use crate::Monitor;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::rc::Rc;

pub type Handler = Box<dyn Fn(&mut Context, &Monitor)>;

pub struct Mainloop {
    pub tx: Rc<spsc::Sender<Context>>,
    pub rx: spsc::Receiver<Context>,
    pub on_event: BTreeMap<usize, Handler>,
}

impl Mainloop {
    pub fn new() -> Self {
        let (tx, rx) = spsc::channel::<Context>();
        Self {
            tx,
            rx,
            on_event: BTreeMap::new(),
        }
    }

    fn add_event_handlers(&mut self) {
        rmi::features::set_event_handler(self);
        rmi::gpt::set_event_handler(self);
        rmi::realm::set_event_handler(self);
        rmi::rec::set_event_handler(self);
        rmi::rtt::set_event_handler(self);
        rmi::version::set_event_handler(self);
    }

    pub fn boot_complete(&mut self, smc: SecureMonitorCall) {
        let ctx = Context {
            cmd: rmi::BOOT_COMPLETE,
            arg: [rmi::BOOT_SUCCESS, 0, 0, 0],
            ..Default::default()
        };
        self.add_event_handlers();
        self.dispatch(smc, ctx);
    }

    pub fn dispatch(&self, smc: SecureMonitorCall, ctx: Context) {
        let ret = smc.call(ctx.cmd, ctx.arg);
        let ctx = Context {
            cmd: ret[0],
            arg: [ret[1], ret[2], ret[3], ret[4]],
            ..Default::default()
        };
        self.tx.send(ctx);
    }

    pub fn run(&self, monitor: &Monitor) {
        loop {
            let mut ctx = self.rx.recv();
            let smc = monitor.smc;

            if self.on_event.is_empty() {
                panic!("There is no registered event handler.");
            }

            match self.on_event.get(&ctx.cmd) {
                Some(handler) => {
                    handler(&mut ctx, monitor);
                    ctx.arg = [ctx.ret[0], ctx.ret[1], ctx.ret[2], ctx.ret[3]];
                }
                None => {
                    error!("Not registered event: {:X}", ctx.cmd);
                    ctx.arg = [rmi::RET_FAIL, 0, 0, 0];
                }
            }

            ctx.cmd = rmi::REQ_COMPLETE;
            self.dispatch(smc, ctx);
        }
    }

    pub fn add_event_handler(&mut self, code: usize, handler: Handler) {
        self.on_event.insert(code, handler);
    }
}
