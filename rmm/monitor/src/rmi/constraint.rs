extern crate alloc;

use crate::event::Command;
use crate::rmi;
use alloc::collections::btree_map::BTreeMap;

#[allow(dead_code)]
#[derive(Default, Copy, Clone)]
pub struct Constraint {
    pub cmd: Command,
    pub arg_num: usize,
    pub ret_num: usize,
    // TODO: add validate function for each RMI command (validate type or value inside registers)
}

impl Constraint {
    pub fn new(cmd: Command, arg_num: usize, ret_num: usize) -> Constraint {
        Constraint {
            cmd,
            arg_num,
            ret_num,
        }
    }
}

lazy_static! {
    static ref CONSTRAINTS: BTreeMap<Command, Constraint> = {
        let mut m = BTreeMap::new();
        m.insert(rmi::VERSION, Constraint::new(rmi::VERSION, 1, 1));
        m.insert(
            rmi::GRANULE_DELEGATE,
            Constraint::new(rmi::GRANULE_DELEGATE, 2, 1),
        );
        m.insert(
            rmi::GRANULE_UNDELEGATE,
            Constraint::new(rmi::GRANULE_UNDELEGATE, 2, 1),
        );
        m.insert(rmi::DATA_CREATE, Constraint::new(rmi::DATA_CREATE, 6, 1));
        m.insert(rmi::DATA_CREATE_UNKNOWN, Constraint::new(rmi::DATA_CREATE_UNKNOWN, 4, 1));
        m.insert(rmi::DATA_DESTROY, Constraint::new(rmi::DATA_DESTROY, 3, 3));
        m.insert(
            rmi::REALM_ACTIVATE,
            Constraint::new(rmi::REALM_ACTIVATE, 2, 1),
        );
        // NOTE: REALM_CREATE has 3 of arg_num and 1 of ret_num according to the specification,
        //       but we're using one more return value for our own purpose.
        m.insert(rmi::REALM_CREATE, Constraint::new(rmi::REALM_CREATE, 3, 2));
        m.insert(
            rmi::REALM_DESTROY,
            Constraint::new(rmi::REALM_DESTROY, 2, 1),
        );
        // NOTE: REC_CREATE has 4 of arg_num and 1 of ret_num according to the specification,
        //       but we're using one more return value for our own purpose.
        m.insert(rmi::REC_CREATE, Constraint::new(rmi::REC_CREATE, 4, 2));
        m.insert(rmi::REC_DESTROY, Constraint::new(rmi::REC_DESTROY, 4, 1));
        m.insert(rmi::REC_ENTER, Constraint::new(rmi::REC_ENTER, 3, 1));
        m.insert(
            rmi::RTT_MAP_UNPROTECTED,
            Constraint::new(rmi::RTT_MAP_UNPROTECTED, 5, 1),
        );
        m.insert(
            rmi::RTT_UNMAP_UNPROTECTED,
            Constraint::new(rmi::RTT_UNMAP_UNPROTECTED, 4, 1),
        );
        m.insert(
            rmi::RTT_READ_ENTRY,
            Constraint::new(rmi::RTT_READ_ENTRY, 4, 5),
        );
        m.insert(rmi::FEATURES, Constraint::new(rmi::FEATURES, 2, 2));
        m.insert(
            rmi::REC_AUX_COUNT,
            Constraint::new(rmi::REC_AUX_COUNT, 2, 2),
        );
        m.insert(
            rmi::RTT_INIT_RIPAS,
            Constraint::new(rmi::RTT_INIT_RIPAS, 4, 2),
        );
        m.insert(
            rmi::RTT_SET_RIPAS,
            Constraint::new(rmi::RTT_SET_RIPAS, 6, 2),
        );
        m.insert(rmi::REQ_COMPLETE, Constraint::new(rmi::REQ_COMPLETE, 4, 2));
        m
    };
}

pub fn validate<T, G>(cmd: Command, ok_func: T, else_func: G)
where
    T: Fn(usize, usize),
    G: FnOnce(), // TODO: error command?
{
    if let Some(c) = CONSTRAINTS.get(&cmd) {
        // TODO: command-specific validation routine if needed
        ok_func(c.arg_num, c.ret_num);
    } else {
        else_func();
    }
}
