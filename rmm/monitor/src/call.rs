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

#[cfg(test)]
pub mod test {
    use alloc::rc::Rc;
    use core::cell::RefCell;

    use super::{Context, Sender};
    use crate::communication::Event;
    use crate::io::Error;

    extern crate alloc;

    struct MockSender {
        data: RefCell<usize>,
    }

    impl MockSender {
        const fn new() -> Self {
            Self {
                data: RefCell::new(0usize),
            }
        }

        fn get(&self) -> usize {
            *self.data.borrow()
        }
    }

    impl Sender for MockSender {
        type Event = usize;

        fn send(&self, event: usize) -> Result<(), Error> {
            self.data.replace(event);
            Ok(())
        }
    }

    #[test]
    fn create_and_reply() {
        let sender = Rc::new(MockSender::new());
        let call = Context::new(1234usize, 5678usize, sender.clone());

        assert_eq!(call.code(), 1234usize);
        assert_eq!(*call.argument(), 5678usize);

        assert!(call.reply(91011usize).is_ok());

        assert_eq!(sender.get(), 91011usize);
    }
}
