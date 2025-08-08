use crate::config::SMCCC_1_3_SVE_HINT;
use crate::event::{Command, Context};
use crate::rmi::constraint::Constraint; // TODO: we might need rsi's own constraint struct in the future
use crate::rsi;

fn pick(cmd: Command) -> Option<Constraint> {
    let constraint = match cmd {
        // XXX: Constraints for RSI and PSCI are not correctly enforced now.
        //      Note that arg and ret values in Context are not used in RSI where
        //      set_reg and get_reg are instead used.
        rsi::ABI_VERSION => Constraint::new(rsi::ABI_VERSION, 2, 3),
        rsi::FEATURES => Constraint::new(rsi::FEATURES, 2, 2),
        rsi::MEASUREMENT_READ => Constraint::new(rsi::MEASUREMENT_READ, 2, 9),
        rsi::MEASUREMENT_EXTEND => Constraint::new(rsi::MEASUREMENT_EXTEND, 11, 1),
        rsi::ATTEST_TOKEN_INIT => Constraint::new(rsi::ATTEST_TOKEN_INIT, 9, 2),
        rsi::ATTEST_TOKEN_CONTINUE => Constraint::new(rsi::ATTEST_TOKEN_CONTINUE, 4, 2),
        rsi::REALM_CONFIG => Constraint::new(rsi::REALM_CONFIG, 2, 1),
        rsi::IPA_STATE_SET => Constraint::new(rsi::IPA_STATE_SET, 5, 3),
        rsi::IPA_STATE_GET => Constraint::new(rsi::IPA_STATE_GET, 3, 3),
        rsi::HOST_CALL => Constraint::new(rsi::HOST_CALL, 2, 1),
        // PSCI
        // XXX: Setting 0 in ret_num currently causes a problem, while PSCI_CPU_SUSPEND,
        //      PSCI_CPU_OFF, PSCI_SYSTEM_OFF, and PSCI_SYSTEM_RESET have no output values.
        rsi::PSCI_VERSION => Constraint::new(rsi::PSCI_VERSION, 1, 1),
        rsi::PSCI_CPU_SUSPEND => Constraint::new(rsi::PSCI_CPU_SUSPEND, 4, 1),
        rsi::PSCI_CPU_OFF => Constraint::new(rsi::PSCI_CPU_OFF, 1, 1),
        rsi::PSCI_CPU_ON => Constraint::new(rsi::PSCI_CPU_ON, 4, 1),
        rsi::PSCI_AFFINITY_INFO => Constraint::new(rsi::PSCI_AFFINITY_INFO, 3, 1),
        rsi::PSCI_SYSTEM_OFF => Constraint::new(rsi::PSCI_SYSTEM_OFF, 1, 1),
        rsi::PSCI_SYSTEM_RESET => Constraint::new(rsi::PSCI_SYSTEM_RESET, 1, 1),

        rsi::PSCI_FEATURES => Constraint::new(rsi::PSCI_FEATURES, 2, 1),
        // XXX: SMCCC_VERSION is not defined in the spec, so remove it if it is not used now
        rsi::SMCCC_VERSION => Constraint::new(rsi::SMCCC_VERSION, 2, 1),
        // XXX: REALM_SEALING_KEY do not exist in the spec
        rsi::ISLET_REALM_SEALING_KEY => Constraint::new(rsi::ISLET_REALM_SEALING_KEY, 2, 5),
        _ => return None,
    };
    Some(constraint)
}

pub fn validate(cmd: Command) -> Context {
    let fid = cmd & !SMCCC_1_3_SVE_HINT;
    let mut ctx = Context::new(fid);
    if cmd & SMCCC_1_3_SVE_HINT != 0 {
        ctx.sve_hint = true;
    }
    if let Some(c) = pick(fid) {
        ctx.resize_ret(c.ret_num);
    } else {
        // rmm.handle_rsi takes care of unregistered command.
        // Just limit the array size here.
        ctx.resize_ret(1);
    }
    ctx
}
