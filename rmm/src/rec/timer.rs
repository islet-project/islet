use super::Rec;
use crate::asm::isb;
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;

use aarch64_cpu::registers::*;

#[cfg(feature = "ns_state_save")]
mod ns_timer {
    use super::*;
    use crate::config::NUM_OF_CPU;
    use crate::cpu::get_cpu_id;
    use crate::rec::context::TimerRegister;
    use core::array::from_fn;
    use lazy_static::lazy_static;
    use spin::mutex::Mutex;

    lazy_static! {
        static ref NS_TIMER: [Mutex<TimerRegister>; NUM_OF_CPU] =
            from_fn(|_| Mutex::new(TimerRegister::default()));
    }

    pub(super) fn restore() {
        let ns_timer = NS_TIMER[get_cpu_id()].lock();
        CNTVOFF_EL2.set(ns_timer.cntvoff_el2);
        CNTPOFF_EL2.set(ns_timer.cntpoff_el2);
        CNTV_CVAL_EL0.set(ns_timer.cntv_cval_el0);
        CNTV_CTL_EL0.set(ns_timer.cntv_ctl_el0);
        CNTP_CVAL_EL0.set(ns_timer.cntp_cval_el0);
        CNTP_CTL_EL0.set(ns_timer.cntp_ctl_el0);
        CNTHCTL_EL2.set(ns_timer.cnthctl_el2);
    }

    pub(super) fn save() {
        let mut timer = NS_TIMER[get_cpu_id()].lock();
        timer.cntvoff_el2 = CNTVOFF_EL2.get();
        timer.cntv_cval_el0 = CNTV_CVAL_EL0.get();
        timer.cntv_ctl_el0 = CNTV_CTL_EL0.get();
        timer.cntpoff_el2 = CNTPOFF_EL2.get();
        timer.cntp_cval_el0 = CNTP_CVAL_EL0.get();
        timer.cntp_ctl_el0 = CNTP_CTL_EL0.get();
        timer.cnthctl_el2 = CNTHCTL_EL2.get();
    }
}

pub fn init_timer(rec: &mut Rec<'_>) {
    let timer = &mut rec.context.timer;
    timer.cnthctl_el2 = (CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET).into();
}

pub fn set_cnthctl(rec: &mut Rec<'_>, val: u64) {
    let timer = &mut rec.context.timer;
    timer.cnthctl_el2 = val;
}

#[cfg(not(fuzzing))]
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

#[cfg(not(fuzzing))]
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

pub fn save_host_state(_rec: &Rec<'_>) {
    #[cfg(feature = "ns_state_save")]
    ns_timer::save();
}

pub fn restore_host_state(_rec: &Rec<'_>) {
    #[cfg(feature = "ns_state_save")]
    ns_timer::restore();
}

// RMM spec A6.2 Realm timers, I<VRWGS>
// On REC entry, for both the EL1 Virtual Timer and the EL1 Physical Timer,
// if the EL1 timer asserts its output in the state described in the REC exit
// structure from the previous REC exit then the RMM masks the hardware timer
// signal before returning to the Realm.
pub fn update_timer_assertion(rec: &mut Rec<'_>) {
    const CNTHCTL_EL2_CNTVMASK: u64 = 0x1 << 18;
    const CNTHCTL_EL2_CNTPMASK: u64 = 0x1 << 19;
    let timer = &mut rec.context.timer;

    // Get recently saved timer control registers
    let cnthctl_old = timer.cnthctl_el2;

    // Check if virtual timer is asserted
    if CNTV_CTL_EL0.matches_all(
        CNTV_CTL_EL0::ISTATUS::SET + CNTV_CTL_EL0::IMASK::CLEAR + CNTV_CTL_EL0::ENABLE::SET,
    ) {
        timer.cnthctl_el2 |= CNTHCTL_EL2_CNTVMASK;
    } else {
        timer.cnthctl_el2 &= !CNTHCTL_EL2_CNTVMASK; // Clear MASK
    }

    // Check if physical timer is asserted
    if CNTP_CTL_EL0.matches_all(
        CNTP_CTL_EL0::ISTATUS::SET + CNTP_CTL_EL0::IMASK::CLEAR + CNTP_CTL_EL0::ENABLE::SET,
    ) {
        timer.cnthctl_el2 |= CNTHCTL_EL2_CNTPMASK;
    } else {
        timer.cnthctl_el2 &= !CNTHCTL_EL2_CNTPMASK; // Clear MASK
    }

    // If cnthctl changed, write it back and ensure synchronization
    if cnthctl_old != timer.cnthctl_el2 {
        CNTHCTL_EL2.set(timer.cnthctl_el2);
        isb();
    }
}
