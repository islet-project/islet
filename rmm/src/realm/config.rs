use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::registry::get_realm;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::rtt::RTT_PAGE_LEVEL;

use safe_abstraction::raw_ptr::assume_safe;

#[repr(C)]
pub struct RealmConfig {
    ipa_width: usize,
}

impl RealmConfig {
    // The below `init()` fills the object allocated in the Realm kernel with the proper
    // value (ipa_width), which helps to redirect the accesses to decrypted pages.
    //
    // For some reason, using 33 for ipa_width causes a problem (format string bug?)
    // in parsing the following kernel cmdline argument:
    // `console=ttyS0 root=/dev/vda rw  console=pl011,mmio,0x1c0a0000 console=ttyAMA0 printk.devkmsg=on`.
    // So, we get back to use the same kernel argument with TF-RMM's one (uart0 & uart3).
    pub fn init(config_addr: usize, ipa_width: usize) -> Result<(), Error> {
        let safety_assumed = assume_safe::<RealmConfig>(config_addr).ok_or(Error::RmiErrorInput)?;
        safety_assumed.mut_with(|config: &mut RealmConfig| config.ipa_width = ipa_width);
        Ok(())
    }
}

pub fn realm_config(id: usize, config_ipa: usize, ipa_bits: usize) -> Result<(), Error> {
    let res = get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pa(GuestPhysAddr::from(config_ipa), RTT_PAGE_LEVEL);
    if let Some(pa) = res {
        RealmConfig::init(pa.into(), ipa_bits)
    } else {
        Err(Error::RmiErrorInput)
    }
}

impl safe_abstraction::raw_ptr::RawPtr for RealmConfig {}

impl safe_abstraction::raw_ptr::SafetyChecked for RealmConfig {}

impl safe_abstraction::raw_ptr::SafetyAssured for RealmConfig {
    fn is_initialized(&self) -> bool {
        // The initialization of this memory is guaranteed
        // according to the RMM Specification A2.2.4 Granule Wiping.
        // This instance belongs to a Data Granule and has been initialized.
        true
    }

    fn verify_ownership(&self) -> bool {
        // The instance's ownership is guaranteed while being processed by the RMM.
        // While the Realm holds RW permissions for the instance,
        // it cannot exercise these permissions from the moment an SMC request is made
        // until the request is completed. Even in multi-core environments,
        // the designated areas are protected by Stage 2 Table,
        // ensuring that there are no adverse effects on RMM's memory safety.
        true
    }
}
