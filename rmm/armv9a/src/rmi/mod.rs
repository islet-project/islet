use alloc::rc::Rc;
use core::cell::RefCell;

use crate::smc;

use monitor::call;
use monitor::communication::{self, Error, ErrorKind};
use monitor::rmi::{self, Code};

extern crate alloc;

pub mod gpt;
pub mod realm;
pub mod version;

pub type Argument = [usize; 7];
pub type Return = usize;
pub type Call = call::Context<Code, Argument, Return>;

pub struct Receiver {
    sender: Rc<Sender>,
}

impl Receiver {
    pub fn new() -> Self {
        Self {
            sender: Rc::new(Sender::new()),
        }
    }
}

impl communication::Receiver for Receiver {
    type Event = Call;

    fn recv(&self) -> Result<Call, Error> {
        let cmd = usize::from(rmi::Code::RequestComplete);
        let arg = self.sender.pop();
        let ret = smc::call(cmd, arg);

        let cmd = ret[0];
        let mut arg = [0usize; 7];
        arg.clone_from_slice(&ret[1..8]);
        Ok(Call::new(Code::from(cmd), arg, self.sender.clone()))
    }
}

pub struct Sender {
    data: RefCell<(usize, [Return; 4])>,
}

impl Sender {
    const fn new() -> Self {
        Self {
            data: RefCell::new((0usize, [0usize; 4])),
        }
    }

    fn pop(&self) -> [Return; 4] {
        let mut d = self.data.borrow_mut();
        let ret = d.1;
        *d = (0usize, [0usize; 4]);
        ret
    }

    fn push(&self, data: usize) -> Result<(), Error> {
        let mut d = self.data.borrow_mut();
        let pos = d.0;
        if pos < 4 {
            d.1[pos] = data;
            d.0 += 1;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::StorageFull))
        }
    }
}

impl communication::Sender for Sender {
    type Event = Return;

    fn send(&self, event: Return) -> Result<(), Error> {
        self.push(event)
    }
}
