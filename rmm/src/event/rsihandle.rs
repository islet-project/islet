extern crate alloc;

use crate::rec::Rec;
use crate::rmi::rec::run::Run;
use crate::rsi;
use crate::rsi::psci;
use crate::Monitor;
// TODO: Change this into rsi::error::Error
use crate::rmi::error::Error;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;

pub type Handler =
    Box<dyn Fn(&[usize], &mut [usize], &Monitor<'_>, &mut Rec<'_>, &mut Run) -> Result<(), Error>>;

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
