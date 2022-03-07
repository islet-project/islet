use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;

use crate::communication::{Event, Receiver};

extern crate alloc;

pub struct Mainloop<T: Receiver>
where
    T::Event: Event,
    <T::Event as Event>::Code: Ord,
{
    receiver: T,
    on_event: BTreeMap<<T::Event as Event>::Code, Box<dyn FnMut(T::Event)>>,
    on_default: Option<Box<dyn FnMut(T::Event)>>,
    on_idle: Option<Box<dyn FnMut()>>,
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
            on_default: None,
            on_idle: None,
        }
    }

    pub fn set_event_handler<F: 'static + FnMut(T::Event)>(
        &mut self,
        code: <T::Event as Event>::Code,
        f: F,
    ) {
        self.on_event.insert(code, Box::new(f));
    }

    pub fn set_default_handler<F: 'static + FnMut(T::Event)>(&mut self, f: F) {
        self.on_default.replace(Box::new(f));
    }

    pub fn set_idle_handler<F: 'static + FnMut()>(&mut self, f: F) {
        self.on_idle.replace(Box::new(f));
    }

    pub fn run(&mut self) {
        for event in self.receiver.iter() {
            match self.on_event.get_mut(&event.code()) {
                Some(f) => f(event),
                _ => {
                    self.on_default.as_mut().map(|f| f(event));
                }
            }
            self.on_idle.as_mut().map(|f| f());
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

        mainloop.set_event_handler(1234usize, move |event| {
            r.replace(true);
            assert_eq!(event.code(), 1234usize);
        });

        mainloop.set_event_handler(91011usize, |_| {
            assert!(false);
        });

        mainloop.run();

        assert!(*result.borrow());
    }

    #[test]
    fn default_handler() {
        let receiver = MockReceiver::new(&[1234usize, 5678usize]);
        let mut mainloop = Mainloop::new(receiver);
        let result = Rc::new(RefCell::new(false));
        let r = result.clone();

        mainloop.set_event_handler(1234usize, |_| {});

        mainloop.set_default_handler(move |event| {
            r.replace(true);
            assert_eq!(event.code(), 5678usize);
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

        mainloop.set_event_handler(5678usize, |_| {});

        mainloop.set_idle_handler(move || (*r.borrow_mut()) += 1);

        mainloop.run();

        assert_eq!(*result.borrow(), 2);
    }
}
