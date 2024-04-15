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

    #[cfg(not(kani))]
    pub fn run(&self) {
        let mut mainloop = Mainloop::new();
        mainloop.boot_complete();
        mainloop.run(self)
    }
    #[cfg(kani)]
    // DIFF: `symbolic` parameter is added to pass symbolic input
    //       return value is added to track output
    pub fn run(&self, symbolic: [usize; 8]) -> [usize; 5] {
        let mut mainloop = Mainloop::new();
        mainloop.boot_complete(symbolic);
        mainloop.run(self)
    }
}
