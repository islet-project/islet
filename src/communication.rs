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
