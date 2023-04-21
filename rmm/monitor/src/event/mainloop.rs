extern crate alloc;

use super::{Context, Handler};
use crate::rmi::{self, RMI};
use crate::rmm::PageMap;
use crate::smc::SecureMonitorCall;
use crate::utils::spsc;

use alloc::collections::btree_map::BTreeMap;
use alloc::rc::Rc;

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

    pub fn dispatch(&self, smc: SecureMonitorCall, ctx: Context) {
        let ret = smc.call(ctx.cmd, ctx.arg);
        let ctx = Context {
            cmd: ret[0],
            arg: [ret[1], ret[2], ret[3], ret[4]],
            ..Default::default()
        };
        self.tx.send(ctx);
    }

    pub fn run(&self, rmi: RMI, smc: SecureMonitorCall, rmm: PageMap) {
        loop {
            let mut ctx = self.rx.recv();

            if self.on_event.is_empty() {
                panic!("There is no registered event handler.");
            }

            match self.on_event.get(&ctx.cmd) {
                Some(handler) => {
                    handler(&mut ctx, rmi, smc, rmm);
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
