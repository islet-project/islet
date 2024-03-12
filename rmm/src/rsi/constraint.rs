extern crate alloc;

use crate::event::Command;
use crate::rmi::constraint::Constraint; // TODO: we might need rsi's own constraint struct in the future
use crate::rsi;
use crate::rsi::psci;
use alloc::collections::btree_map::BTreeMap;

lazy_static! {
    static ref CONSTRAINTS: BTreeMap<Command, Constraint> = {
        let mut m = BTreeMap::new();
        m.insert(
            rsi::IPA_STATE_SET,
            Constraint::new(rsi::IPA_STATE_SET, 2, 1),
        );
        m.insert(rsi::HOST_CALL, Constraint::new(rsi::HOST_CALL, 2, 1));
        m.insert(rsi::ABI_VERSION, Constraint::new(rsi::ABI_VERSION, 2, 1));
        m.insert(rsi::REALM_CONFIG, Constraint::new(rsi::REALM_CONFIG, 2, 1));
        m.insert(
            rsi::IPA_STATE_GET,
            Constraint::new(rsi::IPA_STATE_GET, 2, 1),
        );
        m.insert(
            psci::PSCI_VERSION,
            Constraint::new(psci::PSCI_VERSION, 2, 1),
        );
        m.insert(
            psci::SMC32::CPU_SUSPEND,
            Constraint::new(psci::SMC32::CPU_SUSPEND, 2, 1),
        );
        m.insert(
            psci::SMC64::CPU_SUSPEND,
            Constraint::new(psci::SMC64::CPU_SUSPEND, 2, 1),
        );
        m.insert(
            psci::SMC32::CPU_OFF,
            Constraint::new(psci::SMC32::CPU_OFF, 2, 1),
        );
        m.insert(
            psci::SMC32::CPU_ON,
            Constraint::new(psci::SMC32::CPU_ON, 2, 1),
        );
        m.insert(
            psci::SMC64::CPU_ON,
            Constraint::new(psci::SMC64::CPU_ON, 2, 1),
        );
        m.insert(
            psci::SMC32::AFFINITY_INFO,
            Constraint::new(psci::SMC32::AFFINITY_INFO, 2, 1),
        );
        m.insert(
            psci::SMC64::AFFINITY_INFO,
            Constraint::new(psci::SMC64::AFFINITY_INFO, 2, 1),
        );
        m.insert(
            psci::SMC32::SYSTEM_OFF,
            Constraint::new(psci::SMC32::SYSTEM_OFF, 2, 1),
        );
        m.insert(
            psci::SMC32::SYSTEM_RESET,
            Constraint::new(psci::SMC32::SYSTEM_RESET, 2, 1),
        );
        m.insert(
            psci::SMC32::FEATURES,
            Constraint::new(psci::SMC32::FEATURES, 2, 1),
        );
        m.insert(
            psci::SMCCC_VERSION,
            Constraint::new(psci::SMCCC_VERSION, 2, 1),
        );
        m
    };
}

pub fn validate<T>(cmd: Command, mut ok_func: T)
where
    T: FnMut(usize, usize),
{
    if let Some(c) = CONSTRAINTS.get(&cmd) {
        ok_func(c.arg_num, c.ret_num);
    } else {
        // rsi.dispatch takes care of unregistered command.
        // Just limit the array size here.
        ok_func(2, 1);
    }
}
