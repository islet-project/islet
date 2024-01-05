use super::context::Context;
use crate::realm::registry::get_realm;
use crate::realm::vcpu::VCPU;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::rec::run::Run;

use armv9a::regs::*;

pub fn init_timer(vcpu: &mut VCPU<Context>) {
    let timer = &mut vcpu.context.timer;
    timer.cnthctl_el2 = S3_4_C14_C1_0::EL1PCTEN | S3_4_C14_C1_0::EL1PTEN;
}

pub fn set_cnthctl(vcpu: &mut VCPU<Context>, val: u64) {
    let timer = &mut vcpu.context.timer;
    timer.cnthctl_el2 = val;
}

pub fn restore_state(vcpu: &VCPU<Context>) {
    let timer = &vcpu.context.timer;

    unsafe { CNTVOFF_EL2.set(timer.cntvoff_el2) };
    unsafe { S3_4_C14_C0_6.set(timer.cntpoff_el2) }; // CNTPOFF_EL2
    unsafe { CNTV_CVAL_EL0.set(timer.cntv_cval_el0) };
    unsafe { CNTV_CTL_EL0.set(timer.cntv_ctl_el0) };
    unsafe { CNTP_CVAL_EL0.set(timer.cntp_cval_el0) };
    unsafe { CNTP_CTL_EL0.set(timer.cntp_ctl_el0) };
    unsafe { S3_4_C14_C1_0.set(timer.cnthctl_el2) }; // CNTHCTL_EL2
}

pub fn save_state(vcpu: &mut VCPU<Context>) {
    let timer = &mut vcpu.context.timer;

    timer.cntvoff_el2 = unsafe { CNTVOFF_EL2.get() };
    timer.cntv_cval_el0 = unsafe { CNTV_CVAL_EL0.get() };
    timer.cntv_ctl_el0 = unsafe { CNTV_CTL_EL0.get() };
    timer.cntpoff_el2 = unsafe { S3_4_C14_C0_6.get() }; // CNTPOFF_EL2
    timer.cntp_cval_el0 = unsafe { CNTP_CVAL_EL0.get() };
    timer.cntp_ctl_el0 = unsafe { CNTP_CTL_EL0.get() };
    timer.cnthctl_el2 = unsafe { S3_4_C14_C1_0.get() };
}

pub fn send_state_to_host(id: usize, vcpu: usize, run: &mut Run) -> Result<(), Error> {
    let realm = get_realm(id).ok_or(Error::RmiErrorOthers(NotExistRealm))?;
    let mut locked_realm = realm.lock();
    let vcpu = locked_realm
        .vcpus
        .get_mut(vcpu)
        .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
    let timer = &vcpu.lock().context.timer;

    unsafe {
        run.set_cntv_ctl(timer.cntv_ctl_el0);
        run.set_cntv_cval(timer.cntv_cval_el0 - timer.cntvoff_el2);
        run.set_cntp_ctl(timer.cntp_ctl_el0);
        run.set_cntp_cval(timer.cntp_cval_el0 - timer.cntpoff_el2);
    }
    Ok(())
}
