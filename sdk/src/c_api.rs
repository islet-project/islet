use crate::attester::attest;
use crate::claim::{Claims, Value};
use crate::report::Report;
use crate::sealing::{seal, unseal};
use crate::verifier::verify;

use bincode::{deserialize, serialize};
use std::ffi::{c_char, c_int, c_uchar, CStr};
use std::slice::{from_raw_parts, from_raw_parts_mut};

#[no_mangle]
pub unsafe extern "C" fn islet_attest(
    user_data: *const c_uchar,
    user_data_len: c_int,
    report_out: *mut c_uchar,
    report_out_len: *mut c_int,
) -> c_int {
    let user_data = from_raw_parts(user_data as *const u8, user_data_len as usize);
    let report = attest(user_data).unwrap();
    let encoded = serialize(&report).unwrap();
    *report_out_len = encoded.len() as c_int;
    let out = from_raw_parts_mut(report_out, encoded.len());
    out.copy_from_slice(&encoded[..]);
    0
}

#[no_mangle]
pub unsafe extern "C" fn islet_verify(
    report: *const c_uchar,
    report_len: c_int,
    claims_out: *mut c_uchar,
    claims_out_len: *mut c_int,
) -> c_int {
    let encoded = from_raw_parts(report as *const u8, report_len as usize);
    let decoded: Report = deserialize(encoded).unwrap();

    let claims = verify(&decoded).unwrap();
    let encoded = serialize(&claims).unwrap();
    *claims_out_len = encoded.len() as c_int;
    let out = std::slice::from_raw_parts_mut(claims_out, encoded.len());
    out.copy_from_slice(&encoded[..]);
    0
}

#[no_mangle]
pub unsafe extern "C" fn islet_parse(
    title: *const c_char,
    claims: *const c_uchar,
    claims_len: c_int,
    value_out: *mut c_uchar,
    value_out_len: *mut c_int,
) -> c_int {
    let encoded = from_raw_parts(claims as *const u8, claims_len as usize);
    let decoded: Claims = deserialize(encoded).unwrap();

    let title = CStr::from_ptr(title).to_str().unwrap();
    let value = decoded.value(title).unwrap();
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
        _ => {}
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn islet_print_claims(claims: *const c_uchar, claims_len: c_int) {
    let encoded = from_raw_parts(claims as *const u8, claims_len as usize);
    let decoded: Claims = deserialize(encoded).unwrap();
    println!("{:?}", decoded);
}

#[no_mangle]
pub unsafe extern "C" fn islet_seal(
    plaintext: *const c_uchar,
    plaintext_len: c_int,
    sealed_out: *mut c_uchar,
    sealed_out_len: *mut c_int,
) -> c_int {
    let plaintext = from_raw_parts(plaintext as *const u8, plaintext_len as usize);
    let sealed = seal(plaintext).unwrap();
    *sealed_out_len = sealed.len() as c_int;
    let out = from_raw_parts_mut(sealed_out, sealed.len());
    out.copy_from_slice(&sealed[..]);
    0
}

#[no_mangle]
pub unsafe extern "C" fn islet_unseal(
    sealed: *const c_uchar,
    sealed_len: c_int,
    plaintext_out: *mut c_uchar,
    plaintext_out_len: &mut c_int,
) -> c_int {
    let sealed = from_raw_parts(sealed as *const u8, sealed_len as usize);
    let plaintext = unseal(sealed).unwrap();
    *plaintext_out_len = plaintext.len() as c_int;
    let out = from_raw_parts_mut(plaintext_out, plaintext.len());
    out.copy_from_slice(&plaintext[..]);
    0
}
