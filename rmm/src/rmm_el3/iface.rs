use super::{digest, utils};
use super::{
    ATTEST_KEY_CURVE_ECC_SECP384R1, PLAT_TOKEN, REALM_ATTEST_KEY, RMM_SHARED_BUFFER_LOCK,
    SHA256_DIGEST_SIZE, VHUK_A, VHUK_M,
};
use crate::asm::smc;
use crate::{config, rmi};
use spinning_top::{Spinlock, SpinlockGuard};

const ID_VHUK_A: usize = 0x1;
const ID_VHUK_M: usize = 0x2;

#[derive(Debug)]
enum RmmEl3IfcError {
    Unk,
    BadAddr,
    BadPas,
    NoMem,
    Inval,
}

impl From<isize> for RmmEl3IfcError {
    fn from(value: isize) -> Self {
        match value {
            -1 => RmmEl3IfcError::Unk,
            -2 => RmmEl3IfcError::BadAddr,
            -3 => RmmEl3IfcError::BadPas,
            -4 => RmmEl3IfcError::NoMem,
            -5 => RmmEl3IfcError::Inval,
            _ => panic!("Uknown RMM-EL3 SMC error code"),
        }
    }
}

pub(super) fn get_realm_attest_key() {
    trace!("RMM_GET_REALM_ATTEST_KEY");

    let guard: SpinlockGuard<'_, _> = super::RMM_SHARED_BUFFER_LOCK.lock();

    let ret = smc(
        rmi::RMM_GET_REALM_ATTEST_KEY,
        &[*guard, config::PAGE_SIZE, ATTEST_KEY_CURVE_ECC_SECP384R1],
    );

    let ret_code = ret[0] as isize;
    let buflen = ret[1] as usize;
    debug!(
        "RMM_GET_REALM_ATTEST_KEY returned with: {}, {}",
        ret_code, buflen
    );

    if ret_code != 0 {
        let e: RmmEl3IfcError = ret_code.into();
        error!("RMM_GET_REALM_ATTEST_KEY failed with {:?}", e);
    }

    let v = utils::va_to_vec(*guard, buflen);
    utils::set_vector(v, &REALM_ATTEST_KEY);

    debug!("REALM_ATTEST_KEY: {:02x?}", super::realm_attest_key());
}

pub(super) fn get_plat_token() {
    trace!("RMM_GET_PLAT_TOKEN");

    let guard: SpinlockGuard<'_, _> = RMM_SHARED_BUFFER_LOCK.lock();

    let dak_priv = utils::get_spinlock(&REALM_ATTEST_KEY);
    let dak_pub_hash = digest::get_realm_public_key_hash(dak_priv);
    utils::vec_to_va(&dak_pub_hash, *guard, config::PAGE_SIZE);

    let ret = smc(
        rmi::RMM_GET_PLAT_TOKEN,
        &[*guard, config::PAGE_SIZE, SHA256_DIGEST_SIZE],
    );

    let ret_code = ret[0] as isize;
    let buflen = ret[1] as usize;
    debug!("RMM_GET_PLAT_TOKEN returned with: {}, {}", ret_code, buflen);

    if ret_code != 0 {
        let e: RmmEl3IfcError = ret_code.into();
        error!("RMM_GET_PLAT_TOKEN failed with {:?}", e);
    }

    let v = utils::va_to_vec(*guard, buflen);
    utils::set_vector(v, &PLAT_TOKEN);

    debug!("PLAT_TOKEN: {:02x?}", super::plat_token());
}

pub(super) fn get_vhuks() {
    trace!("RMM_ISLET_GET_VHUK(A&M)");

    let ret_code = get_vhuk(ID_VHUK_A, &VHUK_A);
    if ret_code != 0 {
        let e: RmmEl3IfcError = ret_code.into();
        error!("RMM_ISLET_GET_VHUK(A) failed with {:?}", e);
    }
    debug!("VHUK_A: {:02x?}", super::vhuk_a());

    let ret_code = get_vhuk(ID_VHUK_M, &VHUK_M);
    if ret_code != 0 {
        let e: RmmEl3IfcError = ret_code.into();
        error!("RMM_ISLET_GET_VHUK(M) failed with {:?}", e);
    }
    debug!("VHUK_M: {:02x?}", super::vhuk_m());
}

fn get_vhuk(id: usize, out: &Spinlock<[u8; 32]>) -> isize {
    trace!("RMM_ISLET_GET_VHUK");

    let ret = smc(rmi::RMM_ISLET_GET_VHUK, &[id]);

    let ret_code = ret[0] as isize;
    debug!("RMM_ISLET_GET_VHUK returned with: {}", ret_code);

    if ret_code != 0 {
        let e: RmmEl3IfcError = ret_code.into();
        error!("RMM_ISLET_GET_VHUK failed with {:?}", e);
    }

    utils::set_array(ret, 1..5, out);

    ret_code
}
