extern crate alloc;

use crate::event::Command;
use crate::rmi::constraint::Constraint; // TODO: we might need rsi's own constraint struct in the future
use crate::rsi;
use alloc::collections::btree_map::BTreeMap;

lazy_static! {
    static ref CONSTRAINTS: BTreeMap<Command, Constraint> = {
        let mut m = BTreeMap::new();
        m.insert(
            rsi::IPA_STATE_SET,
            Constraint::new(rsi::IPA_STATE_SET, 4, 2),
        );
        m.insert(rsi::HOST_CALL, Constraint::new(rsi::HOST_CALL, 2, 1));
        m
    };
}

pub fn validate<T, G>(cmd: Command, mut ok_func: T, else_func: G)
where
    T: FnMut(usize, usize),
    G: FnOnce(),
{
    if let Some(c) = CONSTRAINTS.get(&cmd) {
        // TODO: command-specific validation routine if needed
        ok_func(c.arg_num, c.ret_num);
    } else {
        else_func();
    }
}
