use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::registry::get_realm;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::rtt::RTT_PAGE_LEVEL;

#[repr(C)]
pub struct RealmConfig {
    ipa_width: usize,
}

impl RealmConfig {
    #[allow(dead_code)]
    // The below `init()` fills the object allocated in the Realm kernel with the proper
    // value (ipa_width), which helps to redirect the accesses to decrypted pages.
    //
    // For some reason, using 33 for ipa_width causes a problem (format string bug?)
    // in parsing the following kernel cmdline argument:
    // `console=ttyS0 root=/dev/vda rw  console=pl011,mmio,0x1c0a0000 console=ttyAMA0 printk.devkmsg=on`.
    // So, we get back to use the same kernel argument with TF-RMM's one (uart0 & uart3).
    pub unsafe fn init(config_addr: usize, ipa_width: usize) {
        let config: &mut RealmConfig = &mut *(config_addr as *mut RealmConfig);
        config.ipa_width = ipa_width;
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
        let pa: usize = pa.into();
        unsafe { RealmConfig::init(pa, ipa_bits) };
        Ok(())
    } else {
        Err(Error::RmiErrorInput)
    }
}
