use crate::event::{Context, Mainloop, RmiHandle, RsiHandle};
use crate::mm::translation::PageTable;

use crate::rmi;

#[cfg(not(kani))]
pub struct Monitor {
    pub rsi: RsiHandle,
    pub rmi: RmiHandle,
    pub page_table: PageTable,
    mainloop: Mainloop,
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
            rmi: RmiHandle::new(),
            page_table: PageTable::get_ref(),
            mainloop: Mainloop::new(),
        }
    }

    #[cfg(kani)]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(not(kani))]
    fn boot_complete(&self) -> Context {
        let mut ctx = Context::new(rmi::BOOT_COMPLETE);
        ctx.init_arg(&[rmi::BOOT_SUCCESS]);
        self.mainloop.dispatch(ctx)
    }

    #[cfg(kani)]
    // DIFF: `symbolic` parameter is added to pass symbolic input
    pub fn boot_complete(&mut self, symbolic: [usize; 8]) -> Context {
        let mut ctx = Context::new(rmi::BOOT_COMPLETE);
        ctx.init_arg(&[rmi::BOOT_SUCCESS]);
        self.mainloop.dispatch(ctx, symbolic)
    }

    #[cfg(not(kani))]
    pub fn run(&mut self) {
        let mut ctx = self.boot_complete();

        loop {
            match self.rmi.on_event.get(&ctx.cmd) {
                Some(handler) => {
                    ctx.do_rmi(|arg, ret| handler(arg, ret, self));
                }
                None => {
                    error!("Not registered event: {:X}", ctx.cmd);
                    ctx.init_arg(&[rmi::RET_FAIL]);
                }
            };

            ctx.cmd = rmi::REQ_COMPLETE;
            ctx = self.mainloop.dispatch(ctx);
        }
    }

    #[cfg(kani)]
    // DIFF: `symbolic` parameter is added to pass symbolic input
    //       return value is added to track output
    pub fn run(&self, symbolic: [usize; 8]) -> [usize; 5] {
        let mut ctx = self.boot_complete(symbolic);

        match self.rmi.on_event.get(&ctx.cmd) {
            Some(handler) => ctx.do_rmi(|arg, ret| handler(arg, ret, self)),
            None => {
                assert!(false);
                error!("Not registered event: {:X}", ctx.cmd);
                ctx.init_arg(&[rmi::RET_FAIL]);

                return [0; 5]; // this is a bogus statement to meet the return type
            }
        };

        ctx.cmd = rmi::REQ_COMPLETE;
        ctx = self.mainloop.dispatch(ctx, symbolic);
    }
}
