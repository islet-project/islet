/*
 * Single-producer, single-consumer channel
 */

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::default::Default;

pub fn channel<T: Default + Copy>() -> (Rc<Sender<T>>, Receiver<T>) {
    let rx = Receiver::new();
    let tx = rx.sender();
    (tx, rx)
}

pub struct Receiver<T: Default + Copy> {
    sender: Rc<Sender<T>>,
}

impl<T: Default + Copy> Receiver<T> {
    pub fn new() -> Self {
        Self {
            sender: Rc::new(Sender::<T>::new()),
        }
    }

    pub fn sender(&self) -> Rc<Sender<T>> {
        self.sender.clone()
    }

    pub fn recv(&self) -> T {
        self.sender.pop()
    }
}

pub struct Sender<T: Default + Copy> {
    data: RefCell<T>,
}

impl<T: Default + Copy> Sender<T> {
    fn new() -> Self {
        Self {
            data: RefCell::new(Default::default()),
        }
    }

    fn pop(&self) -> T {
        let mut data = self.data.borrow_mut();
        let ret = *data;
        *data = Default::default();
        ret
    }

    fn push(&self, data: T) {
        *self.data.borrow_mut() = data;
    }

    pub fn send(&self, data: T) {
        self.push(data);
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn channel_usize() {
        let (tx, rx) = channel::<usize>();
        tx.send(usize::MIN);
        assert_eq!(rx.recv(), usize::MIN);

        tx.send(usize::MAX);
        assert_eq!(rx.recv(), usize::MAX);
        assert_eq!(rx.recv(), usize::default());
    }

    #[test]
    fn channel_array() {
        let (tx, rx) = channel::<[usize; 2]>();
        tx.send([usize::MIN, usize::MIN]);
        assert_eq!(rx.recv(), [usize::MIN, usize::MIN]);

        tx.send([usize::MAX, usize::MAX]);
        assert_eq!(rx.recv(), [usize::MAX, usize::MAX]);
        assert_eq!(rx.recv(), [usize::default(), usize::default()]);
    }
}
