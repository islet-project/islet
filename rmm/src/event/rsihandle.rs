extern crate alloc;

use super::Context;

use crate::rmi::rec::run::Run;
use crate::rmi::rec::Rec;
use crate::rsi;
use crate::rsi::psci;
use crate::Monitor;
// TODO: Change this into rsi::error::Error
use crate::granule::GranuleState;
use crate::rmi::error::Error;
use crate::rmi::realm::Rd;
use crate::{get_granule, get_granule_if};

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;

pub type Handler =
    Box<dyn Fn(&[usize], &mut [usize], &Monitor, &mut Rec, &mut Run) -> Result<(), Error>>;

pub struct RsiHandle {
    pub on_event: BTreeMap<usize, Handler>,
}

impl RsiHandle {
    pub const RET_SUCCESS: usize = 0x0000;
    pub const RET_FAIL: usize = 0x0001;
    pub const NOT_SUPPORTED: usize = !0;

    pub fn new() -> Self {
        let mut rsi = Self {
            on_event: BTreeMap::new(),
        };
        rsi.set_event_handlers();
        rsi
    }

    pub fn dispatch(
        &self,
        ctx: &mut Context,
        monitor: &Monitor,
        rec: &mut Rec,
        run: &mut Run,
    ) -> usize {
        match self.on_event.get(&ctx.cmd) {
            Some(handler) => {
                ctx.do_rsi(|arg, ret| handler(arg, ret, monitor, rec, run));
            }
            None => {
                let rmi = monitor.rmi;
                let res = get_granule_if!(rec.owner(), GranuleState::RD);
                let g_rd = match res {
                    Ok(g_rd) => g_rd,
                    Err(e) => {
                        error!("failed to get rd: {:?}", e);
                        return RsiHandle::RET_FAIL;
                    }
                };

                let realm_id = g_rd.content::<Rd>().id();
                drop(g_rd); // manually drop to reduce a lock contention

                // TODO: handle the error properly
                let _ = rmi.set_reg(realm_id, rec.id(), 0, RsiHandle::NOT_SUPPORTED);
                error!(
                    "Not registered event: {:X} returning {:X}",
                    ctx.cmd,
                    RsiHandle::NOT_SUPPORTED
                );
                ctx.init_ret(&[RsiHandle::NOT_SUPPORTED]);
                return RsiHandle::RET_FAIL;
            }
        }
        RsiHandle::RET_SUCCESS
    }

    fn set_event_handlers(&mut self) {
        rsi::set_event_handler(self);
        psci::set_event_handler(self);
    }

    pub fn add_event_handler(&mut self, code: usize, handler: Handler) {
        self.on_event.insert(code, handler);
    }
}

unsafe impl Send for RsiHandle {}
unsafe impl Sync for RsiHandle {}
