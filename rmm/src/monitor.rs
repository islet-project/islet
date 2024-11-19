use crate::config::PlatformMemoryLayout;
use crate::event::{Context, Mainloop, RmiHandle, RsiHandle};
use crate::mm::translation::PageTable;
use crate::realm::context::set_reg;
use crate::rec::Rec;
use crate::rmi;
use crate::rmi::rec::run::Run;

#[cfg(not(kani))]
pub struct Monitor<'a> {
    pub rsi: RsiHandle,
    pub rmi: RmiHandle,
    pub page_table: PageTable<'a>,
    mainloop: Mainloop,
}

#[cfg(kani)]
// `rsi` and `page_table` are removed in model checking harnesses
// to reduce overall state space
pub struct Monitor<'a> {
    pub rmi: RmiHandle,
    mainloop: Mainloop,
    phantom: core::marker::PhantomData<&'a bool>,
}

impl<'a> Monitor<'a> {
    #[cfg(not(kani))]
    pub fn new(layout: PlatformMemoryLayout) -> Self {
        Self {
            rsi: RsiHandle::new(),
            rmi: RmiHandle::new(),
            page_table: PageTable::new(layout),
            mainloop: Mainloop::new(),
        }
    }

    #[cfg(kani)]
    pub fn new() -> Self {
        Self {
            rmi: RmiHandle::new(),
            mainloop: Mainloop::new(),
            phantom: core::marker::PhantomData,
        }
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
            self.handle_rmi(&mut ctx);
            ctx = self.mainloop.dispatch(ctx);
        }
    }

    #[cfg(kani)]
    // DIFF: `symbolic` parameter is added to pass symbolic input
    //       return value is added to track output
    //       infinite loop is removed
    pub fn run(&mut self, symbolic: [usize; 8]) -> [usize; 5] {
        let mut ctx = self.boot_complete(symbolic);
        let mut result = [0; 5];

        self.handle_rmi(&mut ctx);
        ctx = self.mainloop.dispatch(ctx, symbolic);
        let ret_len = ctx.ret.len();
        result[..ret_len].copy_from_slice(&ctx.ret[..]);
        result
    }

    pub fn handle_rmi(&mut self, ctx: &mut Context) {
        if let Some(handler) = self.rmi.on_event.get(&ctx.cmd) {
            #[cfg(feature = "stat")]
            {
                if ctx.cmd != rmi::REC_ENTER {
                    trace!("let's get STATS.lock() with cmd {}", rmi::to_str(ctx.cmd));
                    crate::stat::STATS.lock().measure(ctx.cmd, || {
                        if let Err(code) = handler(&ctx.arg[..], &mut ctx.ret[..], self) {
                            self.ret[0] = code.into();
                        }
                    });
                } else if let Err(code) = handler(&ctx.arg[..], &mut ctx.ret[..], self) {
                    ctx.ret[0] = code.into();
                }
            }
            #[cfg(not(feature = "stat"))]
            {
                if let Err(code) = handler(&ctx.arg[..], &mut ctx.ret[..], self) {
                    ctx.ret[0] = code.into();
                }
            }

            trace!(
                "RMI: {0: <20} {1:X?} > {2:X?}",
                rmi::to_str(ctx.cmd),
                &ctx.arg,
                &ctx.ret
            );

            ctx.arg.clear();
            ctx.arg.extend_from_slice(&ctx.ret[..]);

            #[cfg(kani)]
            // the below is a proof helper
            {
                let ret_len = ctx.ret.len();
                #[cfg(any(
                    feature = "mc_rmi_granule_delegate",
                    feature = "mc_rmi_granule_undelegate",
                    feature = "mc_rmi_realm_activate",
                    feature = "mc_rmi_realm_destroy",
                    feature = "mc_rmi_rec_destroy"
                ))]
                assert!(ret_len == 1);
                #[cfg(any(feature = "mc_rmi_rec_aux_count", feature = "mc_rmi_features"))]
                assert!(ret_len == 2);
                #[cfg(feature = "mc_rmi_version")]
                assert!(ret_len == 3);
            }
        } else {
            error!("Not registered event: {:X}", ctx.cmd);
            ctx.init_arg(&[rmi::RET_FAIL]);
        }

        ctx.cmd = rmi::REQ_COMPLETE;
    }

    pub fn handle_rsi(&self, ctx: &mut Context, rec: &mut Rec<'_>, run: &mut Run) -> usize {
        #[cfg(not(kani))]
        match self.rsi.on_event.get(&ctx.cmd) {
            Some(handler) => {
                ctx.do_rsi(|arg, ret| handler(arg, ret, self, rec, run));
            }
            None => {
                ctx.init_ret(&[RsiHandle::NOT_SUPPORTED]);
                error!(
                    "Not registered event: {:X} returning {:X}",
                    ctx.cmd,
                    RsiHandle::NOT_SUPPORTED
                );
                // TODO: handle the error properly
                let _ = set_reg(rec, 0, RsiHandle::NOT_SUPPORTED);

                return RsiHandle::RET_FAIL;
            }
        }
        RsiHandle::RET_SUCCESS
    }
}
