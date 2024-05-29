use crate::gic;
use crate::realm::context::Context;
use crate::realm::rd::Rd;
use crate::realm::registry::VMID_SET;
use crate::realm::timer;
use crate::rmi::error::Error;

use alloc::sync::{Arc, Weak};
use armv9a::bits_in_reg;
use armv9a::regs::*;
use spin::Mutex;

extern crate alloc;

#[repr(C)]
#[derive(Debug)]
pub struct VCPU {
    pub context: Context,
    me: Weak<Mutex<Self>>,
}

impl VCPU {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::<Mutex<Self>>::new_cyclic(|me| {
            Mutex::new(Self {
                me: me.clone(),
                context: Context::new(),
            })
        })
    }

    pub fn into_current(&mut self) {
        unsafe {
            Context::into_current(self);

            core::mem::forget(self.me.upgrade().unwrap());
        }
    }

    pub fn from_current(&mut self) {
        unsafe {
            Context::from_current(self);

            let ptr = Arc::into_raw(self.me.upgrade().unwrap());
            Arc::decrement_strong_count(ptr);
            Arc::from_raw(ptr);
        }
    }

    pub fn is_realm_dead(&self) -> bool {
        // XXX: is this function necessary?
        false
    }
}

impl Drop for VCPU {
    fn drop(&mut self) {
        info!("VCPU dropeed!");
    }
}

pub unsafe fn current() -> Option<&'static mut VCPU> {
    match TPIDR_EL2.get() {
        0 => None,
        current => Some(&mut *(current as *mut VCPU)),
    }
}

pub fn create_vcpu(rd: &mut Rd, mpidr: u64) -> Result<usize, Error> {
    let page_table = rd.s2_table().lock().get_base_address();
    let vttbr = bits_in_reg(VTTBR_EL2::VMID, rd.id() as u64)
        | bits_in_reg(VTTBR_EL2::BADDR, page_table as u64);

    let vcpu = VCPU::new();
    vcpu.lock().context.sys_regs.vttbr = vttbr;
    vcpu.lock().context.sys_regs.vmpidr = mpidr | MPIDR_EL1::RES1;
    timer::init_timer(&mut vcpu.lock());
    gic::init_gic(&mut vcpu.lock());

    rd.vcpus.push(vcpu);
    let vcpuid = rd.vcpus.len() - 1;
    Ok(vcpuid)
}

pub fn remove(id: usize) -> Result<(), Error> {
    VMID_SET
        .lock()
        .remove(&id)
        .then_some(())
        .ok_or(Error::RmiErrorInput)
}
