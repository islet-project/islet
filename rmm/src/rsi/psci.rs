use crate::event::RsiHandle;
use crate::granule::GranuleState;
use crate::listen;
use crate::realm::context::{get_reg, set_reg};
use crate::rmi;
use crate::rmi::realm::{rd::State, Rd};
use crate::rmi::rec::run::Run;
use crate::rmi::rec::Rec;
use crate::Monitor;
use crate::{get_granule, get_granule_if};

pub const SMCCC_VERSION: usize = 0x8000_0000;
pub const SMCCC_ARCH_FEATURES: usize = 0x8000_0001;

pub const PSCI_VERSION: usize = 0x8400_0000;

pub struct SMC32;
impl SMC32 {
    pub const CPU_SUSPEND: usize = 0x8400_0001;
    pub const CPU_OFF: usize = 0x8400_0002;
    pub const CPU_ON: usize = 0x8400_0003;
    pub const AFFINITY_INFO: usize = 0x8400_0004;
    // TODO: commented out for future use
    //pub const MIGRATE: usize = 0x8400_0005;
    //pub const MIGRATE_INFO_TYPE: usize = 0x8400_0006;
    //pub const MIGRATE_INFO_UP_CPU: usize = 0x8400_0007;
    pub const SYSTEM_OFF: usize = 0x8400_0008;
    pub const SYSTEM_RESET: usize = 0x8400_0009;
    pub const FEATURES: usize = 0x8400_000A;
    //pub const CPU_FREEZE: usize = 0x8400_000B;
    //pub const CPU_DEFAULT_SUSPEND: usize = 0x8400_000C;
    //pub const NODE_HW_STATE: usize = 0x8400_000D;
    //pub const SYSTEM_SUSPEND: usize = 0x8400_000E;
    //pub const SET_SUSPEND_MODE: usize = 0x8400_000F;
    // don't know what it is, but linux realm sends this.
    //pub const UNKNOWN:  usize = 0x8400_0050;
}

pub struct SMC64;
impl SMC64 {
    pub const CPU_SUSPEND: usize = 0xC400_0001;
    pub const CPU_ON: usize = 0xC400_0003;
    pub const AFFINITY_INFO: usize = 0xC400_0004;
    //pub const MIGRATE: usize = 0xC400_0005;
    //pub const MIGRATE_INFO_UP_CPU: usize = 0xC400_0007;
    //pub const CPU_DEFAULT_SUSPEND: usize = 0xC400_000C;
    //pub const NODE_HW_STATE: usize = 0xC400_000D;
    //pub const SYSTEM_SUSPEND: usize = 0xC400_000E;
    //pub const SYSTEM_RESET2: usize = 0xC400_0012;
}

struct PsciReturn;
impl PsciReturn {
    const SUCCESS: usize = 0;
    const NOT_SUPPORTED: usize = !0;
    //const INVALID_PARAMS: usize = !1;
    //const DENIED: usize = !2;
    //const ALREADY_ON: usize = !3;
    //const ON_PENDING: usize = !4;
    //const INTERNAL_FAILURE: usize = !5;
    //const NOT_PRESENT: usize = !6;
    //const DISABLED: usize = !7; // UL(-8);
    //const INVALID_ADDRESS: usize = !8; //UL(-9);
}

const SMCCC_MAJOR_VERSION: usize = 1;
const SMCCC_MINOR_VERSION: usize = 2;

const PSCI_MAJOR_VERSION: usize = 1;
const PSCI_MINOR_VERSION: usize = 1;

extern crate alloc;

pub fn set_event_handler(rsi: &mut RsiHandle) {
    let dummy =
        |_arg: &[usize], ret: &mut [usize], _rmm: &Monitor, rec: &mut Rec<'_>, _run: &mut Run| {
            let vcpuid = rec.vcpuid();
            let realmid = rec.realmid()?;

            if set_reg(realmid, vcpuid, 0, PsciReturn::SUCCESS).is_err() {
                warn!(
                    "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                    realmid, vcpuid
                );
            }
            ret[0] = rmi::SUCCESS_REC_ENTER;
            Ok(())
        };

    listen!(rsi, PSCI_VERSION, |_arg, ret, _rmm, rec, _run| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;

        if set_reg(realmid, vcpuid, 0, psci_version()).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, SMC32::CPU_SUSPEND, dummy);
    listen!(rsi, SMC64::CPU_SUSPEND, dummy);
    listen!(rsi, SMC32::CPU_OFF, dummy);
    listen!(rsi, SMC32::CPU_ON, dummy);
    listen!(rsi, SMC64::CPU_ON, dummy);
    listen!(rsi, SMC32::AFFINITY_INFO, dummy);
    listen!(rsi, SMC64::AFFINITY_INFO, dummy);
    listen!(rsi, SMC32::SYSTEM_RESET, dummy);

    listen!(rsi, SMC32::SYSTEM_OFF, |_arg, ret, _rmm, rec, _run| {
        let mut rd = get_granule_if!(rec.owner()?, GranuleState::RD)?;
        let rd = rd.content_mut::<Rd>();
        rd.set_state(State::SystemOff);
        ret[0] = rmi::SUCCESS;
        Ok(())
    });

    listen!(rsi, SMC32::FEATURES, |_arg, ret, _rmm, rec, _run| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;

        let feature_id = get_reg(realmid, vcpuid, 1).unwrap_or(0x0);
        let retval = match feature_id {
            SMC32::CPU_SUSPEND
            | SMC64::CPU_SUSPEND
            | SMC32::CPU_OFF
            | SMC32::CPU_ON
            | SMC64::CPU_ON
            | SMC32::AFFINITY_INFO
            | SMC64::AFFINITY_INFO
            | SMC32::SYSTEM_OFF
            | SMC32::SYSTEM_RESET
            | SMC32::FEATURES
            | SMCCC_VERSION => PsciReturn::SUCCESS,
            _ => PsciReturn::NOT_SUPPORTED,
        };
        if set_reg(realmid, vcpuid, 0, retval).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });

    listen!(rsi, SMCCC_VERSION, |_arg, ret, _rmm, rec, _run| {
        let vcpuid = rec.vcpuid();
        let realmid = rec.realmid()?;

        if set_reg(realmid, vcpuid, 0, smccc_version()).is_err() {
            warn!(
                "Unable to set register 0. realmid: {:?} vcpuid: {:?}",
                realmid, vcpuid
            );
        }
        ret[0] = rmi::SUCCESS_REC_ENTER;
        Ok(())
    });
}

fn psci_version() -> usize {
    (PSCI_MAJOR_VERSION << 16) | PSCI_MINOR_VERSION
}

fn smccc_version() -> usize {
    (SMCCC_MAJOR_VERSION << 16) | SMCCC_MINOR_VERSION
}
