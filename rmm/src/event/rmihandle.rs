extern crate alloc;

use crate::rmi;
use crate::rmi::error::Error;
use crate::Monitor;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;

pub type Handler = Box<dyn Fn(&[usize], &mut [usize], &Monitor<'_>) -> Result<(), Error>>;

pub struct RmiHandle {
    pub on_event: BTreeMap<usize, Handler>,
}

impl RmiHandle {
    pub fn new() -> Self {
        let mut rmi = Self {
            on_event: BTreeMap::new(),
        };
        rmi.add_event_handlers();
        rmi
    }

    #[cfg(not(kani))]
    pub fn add_event_handlers(&mut self) {
        rmi::features::set_event_handler(self);
        rmi::gpt::set_event_handler(self);
        rmi::realm::set_event_handler(self);
        rmi::rec::set_event_handler(self);
        rmi::rtt::set_event_handler(self);
        rmi::version::set_event_handler(self);
    }

    #[cfg(kani)]
    fn add_event_handlers(&mut self) {
        #[cfg(feature = "mc_rmi_features")]
        rmi::features::set_event_handler(self);
        #[cfg(any(
            feature = "mc_rmi_granule_delegate",
            feature = "mc_rmi_granule_undelegate"
        ))]
        rmi::gpt::set_event_handler(self);
        #[cfg(any(
            feature = "mc_rmi_realm_activate",
            feature = "mc_rmi_realm_destroy",
            feature = "mc_rmi_rec_aux_count"
        ))]
        rmi::realm::set_event_handler(self);
        #[cfg(feature = "mc_rmi_rec_destroy")]
        rmi::rec::set_event_handler(self);
        #[cfg(feature = "mc_rmi_version")]
        rmi::version::set_event_handler(self);
    }

    pub fn add_event_handler(&mut self, code: usize, handler: Handler) {
        self.on_event.insert(code, handler);
    }
}
