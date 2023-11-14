use crate::event::{Mainloop, RsiHandle};
use crate::mm::translation::PageTable;

#[cfg(not(kani))]
pub struct Monitor {
    pub rsi: RsiHandle,
    pub page_table: PageTable,
}

#[cfg(kani)]
// `rsi` and `page_table` are removed in model checking harnesses
// to reduce overall state space
pub struct Monitor {}

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
        Self {}
    }

    pub fn run(&self) {
        let mut mainloop = Mainloop::new();
        mainloop.boot_complete();
        mainloop.run(self);
    }
}
