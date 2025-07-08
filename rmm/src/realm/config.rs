use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::rtt::RTT_PAGE_LEVEL;
use crate::realm::rd::{Rd, RPV_SIZE};
use crate::rmi::error::Error;

use safe_abstraction::raw_ptr::assume_safe;

#[repr(C)]
pub struct RealmConfig {
    ipa_width: usize,    // Offset 0x0
    hash_algo: u8,       // Offset 0x8
    dummy: [u8; 0x1F7],  // 0x1F7 == 0x200 - 0x9 (offset 0x8 + 1 byte)
    rpv: [u8; RPV_SIZE], // Offset 0x200
}

impl RealmConfig {
    // The below `init()` fills the object allocated in the Realm kernel with the proper
    // value (ipa_width), which helps to redirect the accesses to decrypted pages.
    //
    // For some reason, using 33 for ipa_width causes a problem (format string bug?)
    // in parsing the following kernel cmdline argument:
    // `console=ttyS0 root=/dev/vda rw  console=pl011,mmio,0x1c0a0000 console=ttyAMA0 printk.devkmsg=on`.
    // So, we get back to use the same kernel argument with TF-RMM's one (uart0 & uart3).
    pub fn init(
        config_addr: usize,
        ipa_width: usize,
        hash_algo: u8,
        rpv: &[u8],
    ) -> Result<(), Error> {
        Ok(assume_safe::<RealmConfig>(config_addr)
            .map(|mut realm_config| realm_config.init_inner(ipa_width, hash_algo, rpv))?)
    }

    fn init_inner(&mut self, ipa_width: usize, hash_algo: u8, rpv: &[u8]) {
        self.ipa_width = ipa_width;
        self.hash_algo = hash_algo;
        self.rpv.copy_from_slice(rpv);
    }
}

pub fn realm_config(rd: &Rd, config_ipa: usize, ipa_bits: usize) -> Result<(), Error> {
    let res = rd
        .s2_table()
        .lock()
        .ipa_to_pa(GuestPhysAddr::from(config_ipa), RTT_PAGE_LEVEL);
    let hash_algo = rd.hash_algo();
    let rpv = rd.personalization_value();
    if let Some(pa) = res {
        RealmConfig::init(pa.into(), ipa_bits, hash_algo, rpv)
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
