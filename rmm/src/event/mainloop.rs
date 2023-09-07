extern crate alloc;

use super::Context;
use crate::asm::smc;
use crate::rmi;
use crate::rmi::error::Error;
use crate::Monitor;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use spin::mutex::Mutex;

pub type Handler = Box<dyn Fn(&[usize], &mut [usize], &Monitor) -> Result<(), Error>>;

pub struct Mainloop {
    pub queue: Mutex<VecDeque<Context>>, // TODO: we need a more realistic queue considering multi-core environments if needed
    pub on_event: BTreeMap<usize, Handler>,
}

impl Mainloop {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            on_event: BTreeMap::new(),
        }
    }

    fn add_event_handlers(&mut self) {
        rmi::features::set_event_handler(self);
        rmi::gpt::set_event_handler(self);
        rmi::realm::set_event_handler(self);
        rmi::rec::set_event_handler(self);
        rmi::rtt::set_event_handler(self);
        rmi::version::set_event_handler(self);
    }

    pub fn boot_complete(&mut self) {
        let mut ctx = Context::new(rmi::BOOT_COMPLETE);
        ctx.init_arg(&[rmi::BOOT_SUCCESS]);

        self.add_event_handlers();
        self.dispatch(ctx);
    }

    pub fn dispatch(&self, ctx: Context) {
        let ret = smc(ctx.cmd(), ctx.arg_slice());
        let cmd = ret[0];

        rmi::constraint::validate(
            cmd,
            |arg_num, ret_num| {
                let mut ctx = Context::new(cmd);
                ctx.init_arg(&ret[1..arg_num]);
                ctx.resize_ret(ret_num);
                self.queue.lock().push_back(ctx);
            },
            || {
                let ctx = Context::new(rmi::NOT_SUPPORTED_YET);
                self.queue.lock().push_back(ctx);
            },
        );
    }

    pub fn run(&self, monitor: &Monitor) {
        loop {
            let mut ctx = self.queue.lock().pop_front().unwrap(); // TODO: remove unwrap here, by introducing a more realistic queue
            if self.on_event.is_empty() {
                panic!("There is no registered event handler.");
            }

            match self.on_event.get(&ctx.cmd) {
                Some(handler) => {
                    ctx.do_rmi(|arg, ret| handler(arg, ret, monitor));
                }
                None => {
                    error!("Not registered event: {:X}", ctx.cmd);
                    ctx.init_arg(&[rmi::RET_FAIL]);
                }
            };

            ctx.cmd = rmi::REQ_COMPLETE;
            self.dispatch(ctx);
        }
    }

    pub fn add_event_handler(&mut self, code: usize, handler: Handler) {
        self.on_event.insert(code, handler);
    }
}
