use crate::event::{Mainloop, RsiHandle};
use crate::mm::translation::PageTable;

pub struct Monitor {
    pub rsi: RsiHandle,
    pub page_table: PageTable,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
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
