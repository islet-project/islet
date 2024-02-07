mod digest;
mod iface;
mod utils;

// TODO: This code should be made in an objective manner with some RMM-EL3
// context but to do that we'd need to have a way to pass this context to the
// main event loop. For the initial version I've decided to modify the original
// code as little as possible.

use crate::asm;
use crate::config;
use alloc::vec::Vec;
use config::RMM_SHARED_BUFFER_START;
use spinning_top::Spinlock;

// TODO: move those consts to a more appropriate place
const SHA256_DIGEST_SIZE: usize = 32;
const ATTEST_KEY_CURVE_ECC_SECP384R1: usize = 0;

static RMM_SHARED_BUFFER_LOCK: Spinlock<usize> = Spinlock::new(RMM_SHARED_BUFFER_START);
static REALM_ATTEST_KEY: Spinlock<Vec<u8>> = Spinlock::new(Vec::new());
static PLAT_TOKEN: Spinlock<Vec<u8>> = Spinlock::new(Vec::new());

pub fn setup_el3_ifc() {
    trace!("Setup EL3 interface");

    asm::dcache_flush(RMM_SHARED_BUFFER_START, config::PAGE_SIZE);
    iface::get_realm_attest_key();
    iface::get_plat_token();
}

#[allow(dead_code)]
pub fn realm_attest_key() -> Vec<u8> {
    utils::get_vector(&REALM_ATTEST_KEY)
}

#[allow(dead_code)]
pub fn plat_token() -> Vec<u8> {
    utils::get_vector(&PLAT_TOKEN)
}
