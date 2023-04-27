extern crate alloc;

use super::Context;

use crate::rmi::rec::run::Run;
use crate::rsi;
use crate::Monitor;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;

pub type Handler = Box<dyn Fn(&mut Context, &Monitor, &mut Run)>;

pub const RET_FAIL: usize = 0x0000;
pub const RET_SUCCESS: usize = 0x0000;

pub struct RsiHandle {
    pub on_event: BTreeMap<usize, Handler>,
}

impl RsiHandle {
    pub fn new() -> Self {
        let mut rsi = Self {
            on_event: BTreeMap::new(),
        };
        rsi.set_event_handlers();
        rsi
    }

    pub fn dispatch(&self, ctx: &mut Context, monitor: &Monitor, run: &mut Run) -> usize {
        match self.on_event.get(&ctx.cmd) {
            Some(handler) => {
                handler(ctx, monitor, run);
                ctx.arg = [ctx.ret[0], ctx.ret[1], ctx.ret[2], ctx.ret[3]];
            }
            None => {
                error!("Not registered event: {:X}", ctx.cmd);
                ctx.arg = [RET_FAIL, 0, 0, 0];
            }
        }
        RET_SUCCESS
    }

    fn set_event_handlers(&mut self) {
        rsi::set_event_handler(self);
    }

    pub fn add_event_handler(&mut self, code: usize, handler: Handler) {
        self.on_event.insert(code, handler);
    }
}

unsafe impl Send for RsiHandle {}
unsafe impl Sync for RsiHandle {}
