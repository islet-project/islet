use super::vm::VM;

use alloc::sync::Weak;
use spin::Mutex;

extern crate alloc;

pub trait Context {
    fn new() -> Self
    where
        Self: Sized;
    unsafe fn set_current(vcpu: &mut VCPU<Self>)
    where
        Self: Sized;
}

#[repr(C)]
#[derive(Debug)]
pub struct VCPU<T: Context> {
    pub context: T,
    pub vm: Weak<Mutex<VM<T>>>, // VM struct the VCPU belongs to
    pub state: State,
    pub pcpu: Option<usize>,
}

impl<T: Context + Default> VCPU<T> {
    pub fn new(vm: Weak<Mutex<VM<T>>>) -> Self {
        Self {
            vm: vm,
            state: State::Init,
            context: T::new(),
            pcpu: None,
        }
    }

    pub fn set_current(&mut self) {
        unsafe { T::set_current(self) }
    }
}

#[derive(Debug)]
pub enum State {
    Init,
    Ready,
    Running,
    Trapped,
    Destroy,
}
