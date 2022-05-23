use super::vm::VM;

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

    fn set_vttbr(vcpu: &mut VCPU<Self>, vmid: u64, pgtlb_baddr: u64)
    where
        Self: Sized;
}

#[repr(C)]
#[derive(Debug)]
pub struct VCPU<T: Context> {
    pub context: T,
    pub state: State,
    pub pcpu: Option<usize>,
    pub vm: Weak<Mutex<VM<T>>>, // VM struct the VCPU belongs to
    me: Weak<Mutex<Self>>,
}

impl<T: Context + Default> VCPU<T> {
    pub fn new(vm: Weak<Mutex<VM<T>>>) -> Arc<Mutex<Self>> {
        Arc::<Mutex<Self>>::new_cyclic(|me| {
            Mutex::new(Self {
                vm: vm,
                me: me.clone(),
                state: State::Stopped,
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
        self.state = State::Stopped;
    }

    pub fn set_vttbr(&mut self, vmid: u64, pgtlb_baddr: u64) {
        T::set_vttbr(self, vmid, pgtlb_baddr);
    }

    pub fn is_vm_dead(&self) -> bool {
        self.vm.strong_count() == 0
    }
}

impl<T: Context> Drop for VCPU<T> {
    fn drop(&mut self) {
        info!("VCPU dropeed!");
    }
}

#[derive(Debug)]
pub enum State {
    Running,
    Stopped,
}
