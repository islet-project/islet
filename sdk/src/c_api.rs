use crate::prelude::*;

use bincode::{deserialize, serialize};
use std::ffi::{c_char, c_int, c_uchar, CStr};
use std::slice::{from_raw_parts, from_raw_parts_mut};

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum islet_status_t {
    ISLET_SUCCESS = 0,
    ISLET_FAILURE = -1,
    ISLET_ERROR_INPUT = -2,
    ISLET_ERROR_WRONG_REPORT = -3,
    ISLET_ERROR_WRONG_CLAIMS = -4,
    ISLET_ERROR_FEATURE_NOT_SUPPORTED = -5,
}

/// Get an attestation report(token).
///
/// # Note
/// This API currently returns hard-coded report to simulate attest operation.
/// In future, this will be finalized to support reports signed by RMM.
/// `User data` could be used as nonce to prevent reply attack.
#[no_mangle]
pub unsafe extern "C" fn islet_attest(
    user_data: *const c_uchar,
    user_data_len: c_int,
    report_out: *mut c_uchar,
    report_out_len: *mut c_int,
) -> islet_status_t {
    if user_data_len > 64 {
        return islet_status_t::ISLET_ERROR_INPUT;
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
        Ok(()) => islet_status_t::ISLET_SUCCESS,
        Err(_) => islet_status_t::ISLET_FAILURE,
    }
}

/// Verify the attestation report and returns attestation claims if succeeded.
#[no_mangle]
pub unsafe extern "C" fn islet_verify(
    report: *const c_uchar,
    report_len: c_int,
    claims_out: *mut c_uchar,
    claims_out_len: *mut c_int,
) -> islet_status_t {
    let do_verify = || -> Result<(), Error> {
        let encoded = from_raw_parts(report as *const u8, report_len as usize);
        let decoded: Report = deserialize(encoded).or(Err(Error::Report))?;

        let _claims = verify(&decoded)?;

        // Encode the report instead of the claims.
        // Because the claims couldn't serialize now.
        let out = std::slice::from_raw_parts_mut(claims_out, encoded.len());
        out.copy_from_slice(&encoded[..]);
        *claims_out_len = out.len() as c_int;
        Ok(())
    };

    match do_verify() {
        Ok(()) => islet_status_t::ISLET_SUCCESS,
        Err(Error::Report) => islet_status_t::ISLET_ERROR_WRONG_REPORT,
        Err(_) => islet_status_t::ISLET_FAILURE,
    }
}

/// Parse the claims with the given title and returns the claim if succeeded.
#[no_mangle]
pub unsafe extern "C" fn islet_parse(
    title: *const c_char,
    claims: *const c_uchar,
    claims_len: c_int,
    value_out: *mut c_uchar,
    value_out_len: *mut c_int,
) -> islet_status_t {
    let do_parse = || -> Result<(), Error> {
        // Actually the report is passed instead of the claims
        // ref. islet_verify()
        let encoded = from_raw_parts(claims as *const u8, claims_len as usize);
        let decoded: Report = deserialize(encoded).or(Err(Error::Report))?;

        let claims = verify(&decoded)?;
        let title = CStr::from_ptr(title).to_str().or(Err(Error::Decoding))?;
        match parse(&claims, title) {
            Some(ClaimData::Bstr(value)) => {
                *value_out_len = value.len() as c_int;
                let out = from_raw_parts_mut(value_out, value.len());
                out.copy_from_slice(&value[..]);
            }
            Some(ClaimData::Text(value)) => {
                let value = value.as_bytes();
                *value_out_len = value.len() as c_int;
                let out = from_raw_parts_mut(value_out, value.len());
                out.copy_from_slice(value);
            }
            None => return Err(Error::Claims),
            _ => return Err(Error::NotSupported),
        };
        Ok(())
    };

    match do_parse() {
        Ok(()) => islet_status_t::ISLET_SUCCESS,
        Err(Error::Claims) => islet_status_t::ISLET_ERROR_WRONG_CLAIMS,
        Err(_) => islet_status_t::ISLET_FAILURE,
    }
}

/// Print all claims including Realm Token and Platform Token.
#[no_mangle]
pub unsafe extern "C" fn islet_print_claims(claims: *const c_uchar, claims_len: c_int) {
    // Actually the report is passed instead of the claims
    // ref. islet_verify()
    let encoded = from_raw_parts(claims as *const u8, claims_len as usize);
    let decoded: Result<Report, Error> = deserialize(encoded).or(Err(Error::Report));
    if decoded.is_err() {
        println!("Wrong claims.");
    }

    match verify(&decoded.unwrap()) {
        Ok(claims) => cca_token::dumper::print_token(&claims),
        Err(error) => println!("Wrong claims {:?}", error),
    }
}

/// Seals the plaintext given into the binary slice
///
/// # Note
/// This API currently seals with a hard-coded key, to simulate seal operation.
/// In future, this will be finalized to support keys derived from HES.
#[no_mangle]
pub unsafe extern "C" fn islet_seal(
    plaintext: *const c_uchar,
    plaintext_len: c_int,
    sealed_out: *mut c_uchar,
    sealed_out_len: *mut c_int,
) -> islet_status_t {
    let do_seal = || -> Result<(), Error> {
        let plaintext = from_raw_parts(plaintext as *const u8, plaintext_len as usize);
        let sealed = seal(plaintext)?;
        *sealed_out_len = sealed.len() as c_int;
        let out = from_raw_parts_mut(sealed_out, sealed.len());
        out.copy_from_slice(&sealed[..]);
        Ok(())
    };

    match do_seal() {
        Ok(()) => islet_status_t::ISLET_SUCCESS,
        Err(_) => islet_status_t::ISLET_FAILURE,
    }
}

/// Unseals into plaintext the sealed binary provided.
///
/// # Note
/// This API currently unseals with a hard-coded key, to simulate unseal operation.
/// In future, this will be finalized to support keys derived from HES.
#[no_mangle]
pub unsafe extern "C" fn islet_unseal(
    sealed: *const c_uchar,
    sealed_len: c_int,
    plaintext_out: *mut c_uchar,
    plaintext_out_len: *mut c_int,
) -> islet_status_t {
    let do_unseal = || -> Result<(), Error> {
        let sealed = from_raw_parts(sealed as *const u8, sealed_len as usize);
        let plaintext = unseal(sealed)?;
        *plaintext_out_len = plaintext.len() as c_int;
        let out = from_raw_parts_mut(plaintext_out, plaintext.len());
        out.copy_from_slice(&plaintext[..]);
        Ok(())
    };

    match do_unseal() {
        Ok(()) => islet_status_t::ISLET_SUCCESS,
        Err(_) => islet_status_t::ISLET_FAILURE,
    }
}
