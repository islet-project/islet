use crate::event::{Command, Context};
use crate::rmi::constraint::Constraint; // TODO: we might need rsi's own constraint struct in the future
use crate::rsi;

fn pick(cmd: Command) -> Option<Constraint> {
    let constraint = match cmd {
        rsi::IPA_STATE_SET => Constraint::new(rsi::IPA_STATE_SET, 2, 1),
        rsi::HOST_CALL => Constraint::new(rsi::HOST_CALL, 2, 1),
        rsi::ABI_VERSION => Constraint::new(rsi::ABI_VERSION, 2, 1),
        rsi::REALM_CONFIG => Constraint::new(rsi::REALM_CONFIG, 2, 1),
        rsi::IPA_STATE_GET => Constraint::new(rsi::IPA_STATE_GET, 2, 1),
        rsi::PSCI_VERSION => Constraint::new(rsi::PSCI_VERSION, 2, 1),
        rsi::PSCI_CPU_SUSPEND => Constraint::new(rsi::PSCI_CPU_SUSPEND, 2, 1),
        rsi::PSCI_CPU_OFF => Constraint::new(rsi::PSCI_CPU_OFF, 2, 1),
        rsi::PSCI_CPU_ON => Constraint::new(rsi::PSCI_CPU_ON, 2, 1),
        rsi::PSCI_AFFINITY_INFO => Constraint::new(rsi::PSCI_AFFINITY_INFO, 2, 1),
        rsi::PSCI_SYSTEM_OFF => Constraint::new(rsi::PSCI_SYSTEM_OFF, 2, 1),
        rsi::PSCI_SYSTEM_RESET => Constraint::new(rsi::PSCI_SYSTEM_RESET, 2, 1),
        rsi::PSCI_FEATURES => Constraint::new(rsi::PSCI_FEATURES, 2, 1),
        rsi::SMCCC_VERSION => Constraint::new(rsi::SMCCC_VERSION, 2, 1),
        rsi::ISLET_REALM_SEALING_KEY => Constraint::new(rsi::ISLET_REALM_SEALING_KEY, 2, 5),
        _ => return None,
    };
    Some(constraint)
}

pub fn validate(cmd: Command) -> Context {
    let mut ctx = Context::new(cmd);
    if let Some(c) = pick(cmd) {
        ctx.resize_ret(c.ret_num);
    } else {
        // rmm.handle_rsi takes care of unregistered command.
        // Just limit the array size here.
        ctx.resize_ret(1);
    }
    ctx
}
