use super::context::Context;
use crate::define_sys_register;
use crate::helper::CNTHCTL_EL2;
use monitor::realm::vcpu::VCPU;

define_sys_register!(CNTVOFF_EL2);
define_sys_register!(CNTV_CVAL_EL0);
define_sys_register!(CNTV_CTL_EL0);
//define_sys_register!(CNTPOFF_EL2);
define_sys_register!(CNTP_CVAL_EL0);
define_sys_register!(CNTP_CTL_EL0);

pub fn init_cnthctl() -> u64 {
    CNTHCTL_EL2::EL1PCTEN | CNTHCTL_EL2::EL1PTEN
}

pub fn set_cnthctl(vcpu: &mut VCPU<Context>, val: u64) {
    let timer = &mut vcpu.context.timer;
    timer.cnthctl_el2 = val;
}

define_sys_register!(CNTVCT_EL0);
define_sys_register!(CNTV_TVAL_EL0);
pub fn restore_state(vcpu: &VCPU<Context>) {
    let timer = &vcpu.context.timer;

    unsafe { CNTVOFF_EL2.set(timer.cntvoff_el2) };
    // FIXME: commented out due to compilation error
    //unsafe { CNTPOFF_EL2.set(timer.cntpoff_el2) };
    unsafe { CNTV_CVAL_EL0.set(timer.cntv_cval_el0) };
    unsafe { CNTV_CTL_EL0.set(timer.cntv_ctl_el0) };
    unsafe { CNTP_CVAL_EL0.set(timer.cntp_cval_el0) };
    unsafe { CNTP_CTL_EL0.set(timer.cntp_ctl_el0) };
    //unsafe { CNTHCTL_EL2.set(timer.cnthctl_el2) };
}

pub fn save_state(vcpu: &mut VCPU<Context>) {
    let timer = &mut vcpu.context.timer;

    *&mut timer.cntvoff_el2 = unsafe { CNTVOFF_EL2.get() };
    *&mut timer.cntv_cval_el0 = unsafe { CNTV_CVAL_EL0.get() };
    *&mut timer.cntv_ctl_el0 = unsafe { CNTV_CTL_EL0.get() };
    // FIXME: commented out due to compilation error
    //*&mut timer.cntpoff_el2 = unsafe { CNTPOFF_EL2.get() };
    *&mut timer.cntp_cval_el0 = unsafe { CNTP_CVAL_EL0.get() };
    *&mut timer.cntp_ctl_el0 = unsafe { CNTP_CTL_EL0.get() };
}
