use crate::event::Command;
use crate::rmi::constraint::Constraint; // TODO: we might need rsi's own constraint struct in the future
use crate::rsi;
use crate::rsi::psci;

fn pick(cmd: Command) -> Option<Constraint> {
    let constraint = match cmd {
        rsi::IPA_STATE_SET => Constraint::new(rsi::IPA_STATE_SET, 2, 1),
        rsi::HOST_CALL => Constraint::new(rsi::HOST_CALL, 2, 1),
        rsi::ABI_VERSION => Constraint::new(rsi::ABI_VERSION, 2, 1),
        rsi::SET_SHRM_TOKEN => Constraint::new(rsi::SET_SHRM_TOKEN, 2, 1),
        rsi::REALM_CONFIG => Constraint::new(rsi::REALM_CONFIG, 2, 1),
        rsi::IPA_STATE_GET => Constraint::new(rsi::IPA_STATE_GET, 2, 1),
        psci::PSCI_VERSION => Constraint::new(psci::PSCI_VERSION, 2, 1),
        psci::SMC32::CPU_SUSPEND => Constraint::new(psci::SMC32::CPU_SUSPEND, 2, 1),
        psci::SMC64::CPU_SUSPEND => Constraint::new(psci::SMC64::CPU_SUSPEND, 2, 1),
        psci::SMC32::CPU_OFF => Constraint::new(psci::SMC32::CPU_OFF, 2, 1),
        psci::SMC32::CPU_ON => Constraint::new(psci::SMC32::CPU_ON, 2, 1),
        psci::SMC64::CPU_ON => Constraint::new(psci::SMC64::CPU_ON, 2, 1),
        psci::SMC32::AFFINITY_INFO => Constraint::new(psci::SMC32::AFFINITY_INFO, 2, 1),
        psci::SMC64::AFFINITY_INFO => Constraint::new(psci::SMC64::AFFINITY_INFO, 2, 1),
        psci::SMC32::SYSTEM_OFF => Constraint::new(psci::SMC32::SYSTEM_OFF, 2, 1),
        psci::SMC32::SYSTEM_RESET => Constraint::new(psci::SMC32::SYSTEM_RESET, 2, 1),
        psci::SMC32::FEATURES => Constraint::new(psci::SMC32::FEATURES, 2, 1),
        psci::SMCCC_VERSION => Constraint::new(psci::SMCCC_VERSION, 2, 1),
        _ => return None,
    };
    Some(constraint)
}

pub fn validate<T>(cmd: Command, mut ok_func: T)
where
    T: FnMut(usize, usize),
{
    if let Some(c) = pick(cmd) {
        ok_func(c.arg_num, c.ret_num);
    } else {
        // rsi.dispatch takes care of unregistered command.
        // Just limit the array size here.
        ok_func(2, 1);
    }
}
