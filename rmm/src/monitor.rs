use crate::event::{Mainloop, RsiHandle};
use crate::mm::translation::PageTable;

#[cfg(not(kani))]
pub struct Monitor {
    pub rsi: RsiHandle,
    pub page_table: PageTable,
}

#[cfg(kani)]
// `rsi` is removed in model checking harnesses
// to reduce overall state space
pub struct Monitor {
    pub page_table: PageTable,
}

impl Monitor {
    #[cfg(not(kani))]
    pub fn new() -> Self {
        Self {
            rsi: RsiHandle::new(),
            page_table: PageTable::get_ref(),
        }
    }
    #[cfg(kani)]
    pub fn new() -> Self {
        Self {
            page_table: PageTable::get_ref(),
        }
    }

    pub fn run(&self) {
        let mut mainloop = Mainloop::new();
        mainloop.boot_complete();
        mainloop.run(self);
    }
}
