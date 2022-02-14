use alloc::rc::Rc;

use crate::communication::{Error, Event, Sender};

extern crate alloc;

pub struct Context<T: Default + Eq, A: Default, R> {
    code: T,
    argument: A,
    sender: Rc<dyn Sender<Event = R>>,
}

impl<T, A, R> Context<T, A, R>
where
    T: Default + Eq,
    A: Default,
{
    pub const fn new(code: T, argument: A, sender: Rc<dyn Sender<Event = R>>) -> Self {
        Self {
            code,
            argument,
            sender,
        }
    }

    pub fn argument(&self) -> &A {
        &self.argument
    }

    pub fn reply(&self, reply: R) -> Result<(), Error> {
        self.sender.send(reply)
    }
}

impl<T, A, R> Event for Context<T, A, R>
where
    T: Default + Copy + Eq,
    A: Default,
{
    type Code = T;

    fn code(&self) -> Self::Code {
        self.code
    }
}
