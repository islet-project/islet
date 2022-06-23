extern crate alloc;

use alloc::collections::btree_map::BTreeMap;

use crate::communication::{Event, Handler, IdleHandler, Receiver};

#[macro_export]
macro_rules! listen {
    ($mainloop:expr, $handler:expr) => {{
        $mainloop.set_idle_handler(alloc::boxed::Box::new($handler))
    }};
    ($mainloop:expr, $code:expr, $handler:expr) => {{
        $mainloop.set_event_handler($code, alloc::boxed::Box::new($handler))
    }};
}

pub struct Mainloop<T: Receiver>
where
    T::Event: Event,
    <T::Event as Event>::Code: Ord,
{
    receiver: T,
    on_event: BTreeMap<<T::Event as Event>::Code, Handler<T::Event>>,
    on_idle: Option<IdleHandler>,
}

impl<T> Mainloop<T>
where
    T: Receiver,
    T::Event: Event,
    <T::Event as Event>::Code: Ord,
{
    pub const fn new(receiver: T) -> Self {
        Self {
            receiver,
            on_event: BTreeMap::new(),
            on_idle: None,
        }
    }

    pub fn set_event_handler(
        &mut self,
        code: <T::Event as Event>::Code,
        handler: Handler<T::Event>,
    ) {
        self.on_event.insert(code, handler);
    }

    pub fn set_idle_handler(&mut self, handler: IdleHandler) {
        self.on_idle.replace(handler);
    }

    pub fn run(&mut self) {
        for event in self.receiver.iter() {
            if self.on_event.is_empty() {
                self.on_idle.as_mut().map(|handler| handler());
            } else {
                match self.on_event.get_mut(&event.code()) {
                    Some(handler) => {
                        if let Err(msg) = handler(event) {
                            error!("Failed on event handler: {}", msg);
                        };
                    }
                    None => {
                        error!("Not registered event.");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use alloc::collections::linked_list::LinkedList;
    use alloc::rc::Rc;
    use core::cell::RefCell;

    use super::{Mainloop, Receiver};
    use crate::io::{Error, ErrorKind};
    use crate::mainloop::Event;

    extern crate alloc;

    struct MockReceiver {
        list: RefCell<LinkedList<usize>>,
    }

    impl MockReceiver {
        pub fn new(codes: &[usize]) -> Self {
            let mut list = LinkedList::new();
            for code in codes {
                list.push_back(*code);
            }

            Self {
                list: RefCell::new(list),
            }
        }
    }

    impl Receiver for MockReceiver {
        type Event = MockEvent;

        fn recv(&self) -> Result<MockEvent, Error> {
            self.list
                .borrow_mut()
                .pop_back()
                .ok_or(Error::new(ErrorKind::NotConnected))
                .map(|code| MockEvent::new(code))
        }
    }

    struct MockEvent {
        code: usize,
    }

    impl MockEvent {
        pub fn new(code: usize) -> Self {
            Self { code }
        }
    }

    impl Event for MockEvent {
        type Code = usize;

        fn code(&self) -> Self::Code {
            self.code
        }
    }

    #[test]
    fn event_handler() {
        let receiver = MockReceiver::new(&[1234usize, 5678usize]);
        let mut mainloop = Mainloop::new(receiver);
        let result = Rc::new(RefCell::new(false));
        let r = result.clone();

        listen!(mainloop, 1234usize, move |event| {
            r.replace(true);
            assert_eq!(event.code(), 1234usize);
            Ok(())
        });

        listen!(mainloop, 91011usize, |_| {
            assert!(false);
            Ok(())
        });

        mainloop.run();

        assert!(*result.borrow());
    }

    #[test]
    fn idle_handler() {
        let receiver = MockReceiver::new(&[1234usize, 5678usize]);
        let mut mainloop = Mainloop::new(receiver);
        let result = Rc::new(RefCell::new(0usize));
        let r = result.clone();

        // If there is no registered event_handler, idle handler is called
        // mainloop.set_event_handler(5678usize, |_| {});

        listen!(mainloop, move || (*r.borrow_mut()) += 1);

        mainloop.run();

        assert_eq!(*result.borrow(), 2);
    }
}
