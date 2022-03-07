pub use crate::error::{Error, ErrorKind};

pub struct Iter<'a, T> {
    receiver: &'a dyn Receiver<Event = T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.receiver.recv().ok()
    }
}

pub trait Event {
    type Code;
    fn code(&self) -> Self::Code;
}

pub trait Receiver {
    type Event;

    fn recv(&self) -> Result<Self::Event, Error>;
    fn iter(&self) -> Iter<'_, Self::Event>
    where
        Self: Sized,
    {
        Iter { receiver: self }
    }
}

pub trait Sender {
    type Event;

    fn send(&self, event: Self::Event) -> Result<(), Error>;
}

#[cfg(test)]
pub mod test {
    use core::cell::RefCell;
    use core::str::Split;

    use super::Receiver;
    use crate::io::{Error, ErrorKind};

    struct MockReceiver<'a> {
        split: RefCell<Split<'a, &'a str>>,
    }

    impl<'a> MockReceiver<'a> {
        pub fn new(string: &'a str) -> Self {
            Self {
                split: RefCell::new(string.split(" ")),
            }
        }
    }

    impl<'a> Receiver for MockReceiver<'a> {
        type Event = &'a str;

        fn recv(&self) -> Result<&'a str, Error> {
            self.split
                .borrow_mut()
                .next()
                .ok_or(Error::new(ErrorKind::NotConnected))
        }
    }

    #[test]
    fn iter() {
        let receiver = MockReceiver::new("Hello world!");

        assert_eq!(receiver.iter().next(), Some("Hello"));
        assert_eq!(receiver.iter().next(), Some("world!"));
        assert_eq!(receiver.iter().next(), None);

        assert_eq!(
            receiver.recv().err().unwrap().kind(),
            ErrorKind::NotConnected
        );
    }
}
