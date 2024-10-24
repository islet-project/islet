use super::Context;
use crate::asm::smc;
use crate::rmi;

pub struct Mainloop;

impl Mainloop {
    pub fn new() -> Self {
        Self
    }

    #[cfg(not(kani))]
    pub fn dispatch(&self, ctx: Context) -> Context {
        let ret = smc(ctx.cmd(), ctx.arg_slice());
        let cmd = ret[0];
        rmi::constraint::validate(cmd, &ret[1..])
    }

    #[cfg(kani)]
    // DIFF: `symbolic` parameter is added to pass symbolic input
    pub fn dispatch(&self, ctx: Context, symbolic: [usize; 8]) -> Context {
        let _ret = smc(ctx.cmd(), ctx.arg_slice());
        let ret = symbolic;
        let cmd = ret[0];
        rmi::constraint::validate(cmd, &ret[1..])
    }
}
