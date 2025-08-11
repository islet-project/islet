mod digest;
mod iface;
mod manifest;
mod utils;

// TODO: This code should be made in an objective manner with some RMM-EL3
// context but to do that we'd need to have a way to pass this context to the
// main event loop. For the initial version I've decided to modify the original
// code as little as possible.

use alloc::vec::Vec;
use spinning_top::{Spinlock, SpinlockGuard};

// TODO: move those consts to a more appropriate place
const SHA256_DIGEST_SIZE: usize = 32;
const ATTEST_KEY_CURVE_ECC_SECP384R1: usize = 0;

const VHUK_LENGTH: usize = 32;

static RMM_SHARED_BUFFER_LOCK: Spinlock<usize> = Spinlock::new(0);
static REALM_ATTEST_KEY: Spinlock<Vec<u8>> = Spinlock::new(Vec::new());
static PLAT_TOKEN: Spinlock<Vec<u8>> = Spinlock::new(Vec::new());
static VHUK_A: Spinlock<[u8; VHUK_LENGTH]> = Spinlock::new([0xAAu8; VHUK_LENGTH]);
static VHUK_M: Spinlock<[u8; VHUK_LENGTH]> = Spinlock::new([0x33u8; VHUK_LENGTH]);

pub fn setup_el3_ifc(el3_shared_buf: u64) {
    trace!("Setup EL3 interface");

    {
        // limit the scope of lock to this scope
        // to avoid spinning forever in the subsequent functions
        let mut guard: SpinlockGuard<'_, _> = RMM_SHARED_BUFFER_LOCK.lock();
        *guard = el3_shared_buf as usize;
    }
    let _ = manifest::load();
    iface::get_realm_attest_key();
    iface::get_plat_token();
    iface::get_vhuks();
}

// TODO: should those functions fail when respective RMM from TF-A failed?

#[allow(dead_code)]
pub fn realm_attest_key() -> Vec<u8> {
    utils::get_spinlock(&REALM_ATTEST_KEY)
}

#[allow(dead_code)]
pub fn plat_token() -> Vec<u8> {
    utils::get_spinlock(&PLAT_TOKEN)
}

#[allow(dead_code)]
pub fn vhuk_a() -> [u8; VHUK_LENGTH] {
    utils::get_spinlock(&VHUK_A)
}

#[allow(dead_code)]
pub fn vhuk_m() -> [u8; VHUK_LENGTH] {
    utils::get_spinlock(&VHUK_M)
}
