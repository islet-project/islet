extern crate alloc;

use super::Context;

use crate::rmi::rec::run::Run;
use crate::rmi::rec::Rec;
use crate::rsi;
use crate::rsi::psci;
use crate::Monitor;
// TODO: Change this into rsi::error::Error
use crate::realm::context::set_reg;
use crate::rmi::error::Error;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;

pub type Handler =
    Box<dyn Fn(&[usize], &mut [usize], &Monitor, &mut Rec<'_>, &mut Run) -> Result<(), Error>>;

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
        rec: &mut Rec<'_>,
        run: &mut Run,
    ) -> usize {
        match self.on_event.get(&ctx.cmd) {
            Some(handler) => {
                ctx.do_rsi(|arg, ret| handler(arg, ret, monitor, rec, run));
            }
            None => {
                ctx.init_ret(&[RsiHandle::NOT_SUPPORTED]);
                error!(
                    "Not registered event: {:X} returning {:X}",
                    ctx.cmd,
                    RsiHandle::NOT_SUPPORTED
                );

                let realm_id = match rec.realmid() {
                    Ok(realm_id) => realm_id,
                    Err(e) => {
                        error!("Failed to get realmid with {:?}", e);
                        return RsiHandle::RET_FAIL;
                    }
                };

                // TODO: handle the error properly
                let _ = set_reg(realm_id, rec.vcpuid(), 0, RsiHandle::NOT_SUPPORTED);

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
