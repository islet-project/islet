use super::Realm;

use alloc::sync::{Arc, Weak};
use spin::Mutex;

extern crate alloc;

pub trait Context {
    fn new() -> Self
    where
        Self: Sized;

    unsafe fn into_current(vcpu: &mut VCPU<Self>)
    where
        Self: Sized;

    unsafe fn from_current(vcpu: &mut VCPU<Self>)
    where
        Self: Sized;
}

#[repr(C)]
#[derive(Debug)]
pub struct VCPU<T: Context> {
    pub context: T,
    pub state: State,
    pub pcpu: Option<usize>,
    pub realm: Arc<Mutex<Realm<T>>>, // Realm struct the VCPU belongs to
    me: Weak<Mutex<Self>>,
}

impl<T: Context + Default> VCPU<T> {
    pub fn new(realm: Arc<Mutex<Realm<T>>>) -> Arc<Mutex<Self>> {
        Arc::<Mutex<Self>>::new_cyclic(|me| {
            Mutex::new(Self {
                realm: realm,
                me: me.clone(),
                state: State::Ready,
                context: T::new(),
                pcpu: None,
            })
        })
    }

    pub fn into_current(&mut self) {
        unsafe {
            T::into_current(self);

            core::mem::forget(self.me.upgrade().unwrap());
        }
        self.state = State::Running;
    }

    pub fn from_current(&mut self) {
        unsafe {
            T::from_current(self);

            let ptr = Arc::into_raw(self.me.upgrade().unwrap());
            Arc::decrement_strong_count(ptr);
            Arc::from_raw(ptr);
        }
        self.state = State::Ready;
    }

    pub fn is_realm_dead(&self) -> bool {
        Arc::strong_count(&self.realm) == 0
    }
}

impl<T: Context> Drop for VCPU<T> {
    fn drop(&mut self) {
        info!("VCPU dropeed!");
    }
}

#[derive(Debug)]
pub enum State {
    Null,
    Ready,
    Running,
}
