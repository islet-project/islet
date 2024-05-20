use crate::realm::rd::Rd;
use crate::realm::vcpu::VCPU;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::rec::run::Run;

use armv9a::regs::*;

pub fn init_timer(vcpu: &mut VCPU) {
    let timer = &mut vcpu.context.timer;
    timer.cnthctl_el2 = CNTHCTL_EL2::EL1PCTEN | CNTHCTL_EL2::EL1PTEN;
}

pub fn set_cnthctl(vcpu: &mut VCPU, val: u64) {
    let timer = &mut vcpu.context.timer;
    timer.cnthctl_el2 = val;
}

pub fn restore_state(vcpu: &VCPU) {
    let timer = &vcpu.context.timer;

    unsafe { CNTVOFF_EL2.set(timer.cntvoff_el2) };
    unsafe { CNTPOFF_EL2.set(timer.cntpoff_el2) };
    unsafe { CNTV_CVAL_EL0.set(timer.cntv_cval_el0) };
    unsafe { CNTV_CTL_EL0.set(timer.cntv_ctl_el0) };
    unsafe { CNTP_CVAL_EL0.set(timer.cntp_cval_el0) };
    unsafe { CNTP_CTL_EL0.set(timer.cntp_ctl_el0) };
    unsafe { CNTHCTL_EL2.set(timer.cnthctl_el2) };
}

pub fn save_state(vcpu: &mut VCPU) {
    let timer = &mut vcpu.context.timer;

    timer.cntvoff_el2 = unsafe { CNTVOFF_EL2.get() };
    timer.cntv_cval_el0 = unsafe { CNTV_CVAL_EL0.get() };
    timer.cntv_ctl_el0 = unsafe { CNTV_CTL_EL0.get() };
    timer.cntpoff_el2 = unsafe { CNTPOFF_EL2.get() };
    timer.cntp_cval_el0 = unsafe { CNTP_CVAL_EL0.get() };
    timer.cntp_ctl_el0 = unsafe { CNTP_CTL_EL0.get() };
    timer.cnthctl_el2 = unsafe { CNTHCTL_EL2.get() };
}

pub fn send_state_to_host(rd: &mut Rd, vcpu: usize, run: &mut Run) -> Result<(), Error> {
    let vcpu = rd
        .vcpus
        .get_mut(vcpu)
        .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
    let timer = &vcpu.lock().context.timer;

    run.set_cntv_ctl(timer.cntv_ctl_el0);
    run.set_cntv_cval(timer.cntv_cval_el0 - timer.cntvoff_el2);
    run.set_cntp_ctl(timer.cntp_ctl_el0);
    run.set_cntp_cval(timer.cntp_cval_el0 - timer.cntpoff_el2);
    Ok(())
}
