use super::Realm;

use crate::gic;
use crate::realm::registry::get_realm;
use crate::realm::registry::RMS;
use crate::realm::timer;
use crate::rmi::error::Error;
use alloc::sync::{Arc, Weak};
use armv9a::bits_in_reg;
use armv9a::regs::*;
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
                realm,
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

#[derive(Copy, Clone, Debug)]
pub enum State {
    Null = 0,
    Ready = 1,
    Running = 2,
}

pub unsafe fn current() -> Option<&'static mut VCPU<crate::realm::context::Context>> {
    match TPIDR_EL2.get() {
        0 => None,
        current => Some(&mut *(current as *mut VCPU<crate::realm::context::Context>)),
    }
}

pub fn create_vcpu(id: usize) -> Result<usize, Error> {
    let realm = get_realm(id).ok_or(Error::RmiErrorInput)?;

    let page_table = realm.lock().page_table.lock().get_base_address();
    let vttbr =
        bits_in_reg(VTTBR_EL2::VMID, id as u64) | bits_in_reg(VTTBR_EL2::BADDR, page_table as u64);

    let vcpu = VCPU::new(realm.clone());
    vcpu.lock().context.sys_regs.vttbr = vttbr;
    timer::init_timer(&mut vcpu.lock());
    gic::init_gic(&mut vcpu.lock());

    realm.lock().vcpus.push(vcpu);
    let vcpuid = realm.lock().vcpus.len() - 1;
    Ok(vcpuid)
}

pub fn remove(id: usize) -> Result<(), Error> {
    RMS.lock().1.remove(&id).ok_or(Error::RmiErrorInput)?;
    Ok(())
}
