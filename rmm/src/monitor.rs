use crate::event::{Mainloop, RsiHandle};
use crate::mm::translation::PageTable;
use crate::realm;
use crate::rmi::RMI;

pub struct Monitor {
    pub rmi: RMI,
    pub rsi: RsiHandle,
    pub page_table: PageTable,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            rmi: realm::registry::RMI::new(),
            rsi: RsiHandle::new(),
            page_table: PageTable::get_ref(),
        }
    }

    pub fn run(&self) {
        let mut mainloop = Mainloop::new();
        mainloop.boot_complete();
        mainloop.run(self);
    }
}
