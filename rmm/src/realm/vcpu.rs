use crate::realm::rd::Rd;
use crate::realm::registry::VMID_SET;
use crate::rec::Rec;
use crate::rmi::error::Error;

use alloc::sync::{Arc, Weak};
use armv9a::bits_in_reg;
use armv9a::regs::*;
use spin::Mutex;

extern crate alloc;

#[repr(C)]
#[derive(Debug)]
pub struct VCPU {
    me: Weak<Mutex<Self>>,
}

impl VCPU {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::<Mutex<Self>>::new_cyclic(|me| Mutex::new(Self { me: me.clone() }))
    }
}

impl Drop for VCPU {
    fn drop(&mut self) {
        info!("VCPU dropeed!");
    }
}

// XXX: is using 'static okay here?
pub unsafe fn current() -> Option<&'static mut Rec<'static>> {
    match TPIDR_EL2.get() {
        0 => None,
        current => Some(&mut *(current as *mut Rec<'_>)),
    }
}

pub fn create_vcpu(rd: &mut Rd, mpidr: u64) -> Result<(usize, u64, u64), Error> {
    let page_table = rd.s2_table().lock().get_base_address();
    let vttbr = bits_in_reg(VTTBR_EL2::VMID, rd.id() as u64)
        | bits_in_reg(VTTBR_EL2::BADDR, page_table as u64);
    let vmpidr = mpidr | MPIDR_EL1::RES1;

    let vcpu = VCPU::new();

    rd.vcpus.push(vcpu);
    let vcpuid = rd.vcpus.len() - 1;
    Ok((vcpuid, vttbr, vmpidr))
}

pub fn remove(id: usize) -> Result<(), Error> {
    VMID_SET
        .lock()
        .remove(&id)
        .then_some(())
        .ok_or(Error::RmiErrorInput)
}
