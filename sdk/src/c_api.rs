use crate::attester::attest;
use crate::claim::{Claims, Value};
use crate::error::Error;
use crate::report::Report;
use crate::sealing::{seal, unseal};
use crate::verifier::verify;

use bincode::{deserialize, serialize};
use std::ffi::{c_char, c_int, c_uchar, CStr};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub const ISLET_E_SUCCESS: c_int = 0;
pub const ISLET_E_FAILURE: c_int = -1;
pub const ISLET_E_INPUT: c_int = -2;
pub const ISLET_E_WRONG_REPORT: c_int = -3;
pub const ISLET_E_WRONG_CLAIMS: c_int = -4;
pub const ISLET_E_NOT_SUPPORTED: c_int = -5;

#[no_mangle]
pub unsafe extern "C" fn islet_attest(
    user_data: *const c_uchar,
    user_data_len: c_int,
    report_out: *mut c_uchar,
    report_out_len: *mut c_int,
) -> c_int {
    if user_data_len > 64 {
        return ISLET_E_INPUT;
    }

    let do_attest = || -> Result<(), Error> {
        let user_data = from_raw_parts(user_data as *const u8, user_data_len as usize);
        let report = attest(user_data)?;
        let encoded = serialize(&report).or(Err(Error::Serialize))?;
        *report_out_len = encoded.len() as c_int;
        let out = from_raw_parts_mut(report_out, encoded.len());
        out.copy_from_slice(&encoded[..]);
        Ok(())
    };

    match do_attest() {
        Ok(()) => ISLET_E_SUCCESS,
        Err(_) => ISLET_E_FAILURE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn islet_verify(
    report: *const c_uchar,
    report_len: c_int,
    claims_out: *mut c_uchar,
    claims_out_len: *mut c_int,
) -> c_int {
    let do_verify = || -> Result<(), Error> {
        let encoded = from_raw_parts(report as *const u8, report_len as usize);
        let decoded: Report = deserialize(encoded).or(Err(Error::Report))?;

        let claims = verify(&decoded)?;
        let encoded = serialize(&claims).or(Err(Error::Serialize))?;
        *claims_out_len = encoded.len() as c_int;
        let out = std::slice::from_raw_parts_mut(claims_out, encoded.len());
        out.copy_from_slice(&encoded[..]);
        Ok(())
    };

    match do_verify() {
        Ok(()) => ISLET_E_SUCCESS,
        Err(Error::Report) => ISLET_E_WRONG_REPORT,
        Err(_) => ISLET_E_FAILURE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn islet_parse(
    title: *const c_char,
    claims: *const c_uchar,
    claims_len: c_int,
    value_out: *mut c_uchar,
    value_out_len: *mut c_int,
) -> c_int {
    let do_parse = || -> Result<(), Error> {
        let encoded = from_raw_parts(claims as *const u8, claims_len as usize);
        let decoded: Claims = deserialize(encoded).or(Err(Error::Claims))?;

        let title = CStr::from_ptr(title).to_str().or(Err(Error::Decoding))?;
        let value = decoded.value(title).ok_or(Error::Claims)?;
        match value {
            Value::Bytes(value) => {
                *value_out_len = value.len() as c_int;
                let out = from_raw_parts_mut(value_out, value.len());
                out.copy_from_slice(&value[..]);
            }
            Value::String(value) => {
                let value = value.as_bytes();
                *value_out_len = value.len() as c_int;
                let out = from_raw_parts_mut(value_out, value.len());
                out.copy_from_slice(value);
            }
            _ => return Err(Error::NotSupported),
        };
        Ok(())
    };

    match do_parse() {
        Ok(()) => ISLET_E_SUCCESS,
        Err(Error::Claims) => ISLET_E_WRONG_CLAIMS,
        Err(_) => ISLET_E_FAILURE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn islet_print_claims(claims: *const c_uchar, claims_len: c_int) {
    let encoded = from_raw_parts(claims as *const u8, claims_len as usize);
    if let Ok(decoded) = deserialize::<Claims>(encoded) {
        println!("{:?}", decoded);
    }
}

#[no_mangle]
pub unsafe extern "C" fn islet_seal(
    plaintext: *const c_uchar,
    plaintext_len: c_int,
    sealed_out: *mut c_uchar,
    sealed_out_len: *mut c_int,
) -> c_int {
    let do_seal = || -> Result<(), Error> {
        let plaintext = from_raw_parts(plaintext as *const u8, plaintext_len as usize);
        let sealed = seal(plaintext)?;
        *sealed_out_len = sealed.len() as c_int;
        let out = from_raw_parts_mut(sealed_out, sealed.len());
        out.copy_from_slice(&sealed[..]);
        Ok(())
    };

    match do_seal() {
        Ok(()) => ISLET_E_SUCCESS,
        Err(_) => ISLET_E_FAILURE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn islet_unseal(
    sealed: *const c_uchar,
    sealed_len: c_int,
    plaintext_out: *mut c_uchar,
    plaintext_out_len: *mut c_int,
) -> c_int {
    let do_unseal = || -> Result<(), Error> {
        let sealed = from_raw_parts(sealed as *const u8, sealed_len as usize);
        let plaintext = unseal(sealed)?;
        *plaintext_out_len = plaintext.len() as c_int;
        let out = from_raw_parts_mut(plaintext_out, plaintext.len());
        out.copy_from_slice(&plaintext[..]);
        Ok(())
    };

    match do_unseal() {
        Ok(()) => ISLET_E_SUCCESS,
        Err(_) => ISLET_E_FAILURE,
    }
}
