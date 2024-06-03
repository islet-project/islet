use crate::rec::Rec;
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;

use aarch64_cpu::registers::*;

pub fn init_timer(rec: &mut Rec<'_>) {
    let timer = &mut rec.context.timer;
    timer.cnthctl_el2 = (CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET).into();
}

pub fn set_cnthctl(rec: &mut Rec<'_>, val: u64) {
    let timer = &mut rec.context.timer;
    timer.cnthctl_el2 = val;
}

pub fn restore_state(rec: &Rec<'_>) {
    let timer = &rec.context.timer;

    CNTVOFF_EL2.set(timer.cntvoff_el2);
    CNTPOFF_EL2.set(timer.cntpoff_el2);
    CNTV_CVAL_EL0.set(timer.cntv_cval_el0);
    CNTV_CTL_EL0.set(timer.cntv_ctl_el0);
    CNTP_CVAL_EL0.set(timer.cntp_cval_el0);
    CNTP_CTL_EL0.set(timer.cntp_ctl_el0);
    CNTHCTL_EL2.set(timer.cnthctl_el2);
}

pub fn save_state(rec: &mut Rec<'_>) {
    let timer = &mut rec.context.timer;

    timer.cntvoff_el2 = CNTVOFF_EL2.get();
    timer.cntv_cval_el0 = CNTV_CVAL_EL0.get();
    timer.cntv_ctl_el0 = CNTV_CTL_EL0.get();
    timer.cntpoff_el2 = CNTPOFF_EL2.get();
    timer.cntp_cval_el0 = CNTP_CVAL_EL0.get();
    timer.cntp_ctl_el0 = CNTP_CTL_EL0.get();
    timer.cnthctl_el2 = CNTHCTL_EL2.get();
}

pub fn send_state_to_host(rec: &Rec<'_>, run: &mut Run) -> Result<(), Error> {
    let timer = &rec.context.timer;

    run.set_cntv_ctl(timer.cntv_ctl_el0);
    run.set_cntv_cval(timer.cntv_cval_el0 - timer.cntvoff_el2);
    run.set_cntp_ctl(timer.cntp_ctl_el0);
    run.set_cntp_cval(timer.cntp_cval_el0 - timer.cntpoff_el2);
    Ok(())
}
