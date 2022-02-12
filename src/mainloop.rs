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
