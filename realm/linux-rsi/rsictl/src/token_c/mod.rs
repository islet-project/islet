#[allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
)]
pub(crate) mod bindgen;

#[derive(Debug)]
pub enum TokenError
{
    InitError,
    MissingMandatoryClaim,
    InvalidCoseTag,
    InvalidClaimLen,
    InternalError,
}

impl std::fmt::Display for TokenError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TokenError {}

impl From<i32> for TokenError
{
    fn from(value: i32) -> Self
    {
        match value {
            1 => TokenError::InitError,
            2 => TokenError::MissingMandatoryClaim,
            3 => TokenError::InvalidCoseTag,
            4 => TokenError::InvalidClaimLen,
            _ => TokenError::InternalError,
        }
    }
}

fn new_claims() -> bindgen::attestation_claims
{
    let claim_union = bindgen::claim_t__bindgen_ty_1 {
        bool_data: false,
    };
    let claim = bindgen::claim_t {
        mandatory: false,
        type_: 0,
        key: 0,
        title: std::ptr::null() as *const std::os::raw::c_char,
        present: false,
        __bindgen_anon_1: claim_union,
    };
    let component = bindgen::sw_component_t {
        present: false,
        claims: [claim; bindgen::CLAIM_COUNT_SW_COMPONENT as usize],
    };
    bindgen::attestation_claims {
        realm_cose_sign1_wrapper: [claim; bindgen::CLAIM_COUNT_COSE_SIGN1_WRAPPER as usize],
        realm_token_claims: [claim; bindgen::CLAIM_COUNT_REALM_TOKEN as usize],
        realm_measurement_claims: [claim; bindgen::CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS as usize],
        plat_cose_sign1_wrapper: [claim; bindgen::CLAIM_COUNT_COSE_SIGN1_WRAPPER as usize],
        plat_token_claims: [claim; bindgen::CLAIM_COUNT_PLATFORM_TOKEN as usize],
        sw_component_claims: [component; bindgen::MAX_SW_COMPONENT_COUNT as usize]
    }
}

pub fn verify_token(token: &[u8])
                    -> Result<bindgen::attestation_claims, TokenError>
{
    let mut claims = new_claims();
    let ret = unsafe {
        bindgen::verify_token(token.as_ptr() as *const std::os::raw::c_char,
                                token.len(), &mut claims)
    };
    match ret {
        0 => Ok(claims),
        _ => Err(ret.into()),
    }
}

#[allow(dead_code)]
pub fn print_raw_token(token: &[u8])
{
    unsafe {
        bindgen::print_raw_token(token.as_ptr() as *const std::os::raw::c_char,
                                   token.len());
    }
}

pub fn print_token(claims: &bindgen::attestation_claims)
{
    unsafe {
        bindgen::print_token(claims as *const bindgen::attestation_claims);
    }
}

// Rust code that prints c struct

use std::ffi::{CStr, c_char};
use core::slice;

const COLUMN: usize = 30;

fn cstr_to_str<'a>(s: *const c_char) -> &'a str
{
    unsafe {
        CStr::from_ptr(s)
    }.to_str().unwrap()
}

fn print_indent(indent: i32)
{
    for _i in 0..indent {
        print!("  ");
    }
}

fn print_byte_string(name: *const c_char, index: i64,
                     buf: bindgen::q_useful_buf_c)
{
    let v = unsafe {
        slice::from_raw_parts(buf.ptr as *const u8, buf.len)
    }.to_vec();
    println!("{:COLUMN$} (#{}) = [{}]", cstr_to_str(name), index, hex::encode(v));
}

fn print_text(name: *const c_char, index: i64,
              buf: bindgen::q_useful_buf_c)
{
    let v = unsafe {
        slice::from_raw_parts(buf.ptr as *const u8, buf.len)
    }.to_vec();
    println!("{:COLUMN$} (#{}) = \"{}\"", cstr_to_str(name), index, String::from_utf8_lossy(&v));
}

fn print_claim(claim: &bindgen::claim_t, indent: i32)
{
    print_indent(indent);

    if claim.present {
        match claim.type_ {
            bindgen::claim_data_type_CLAIM_INT64 =>
                println!("{:COLUMN$} (#{}) = {}",
                         cstr_to_str(claim.title), claim.key,
                         unsafe { claim.__bindgen_anon_1.int_data }),
            bindgen::claim_data_type_CLAIM_BOOL =>
                println!("{:COLUMN$} (#{}) = {}",
                         cstr_to_str(claim.title), claim.key,
                         unsafe { claim.__bindgen_anon_1.bool_data }),
            bindgen::claim_data_type_CLAIM_BSTR =>
                print_byte_string(claim.title, claim.key,
                                  unsafe { claim.__bindgen_anon_1.buffer_data }),
            bindgen::claim_data_type_CLAIM_TEXT =>
                print_text(claim.title, claim.key,
                           unsafe { claim.__bindgen_anon_1.buffer_data }),
            _ => println!("* Internal error, print_claim, Key: {}, Title: {}",
                          claim.key, cstr_to_str(claim.title)),
        }
    } else {
        let mandatory = if claim.mandatory { "mandatory " } else { "" };
        println!("* Missing {}claim with key: {} ({})",
                 mandatory, claim.key, cstr_to_str(claim.title));
    }
}

fn print_cose_sign1_wrapper(token_type: &str,
                            cose_sign1_wrapper: &[bindgen::claim_t])
{
    println!("== {} Token cose header:", token_type);
    print_claim(&cose_sign1_wrapper[0], 0);
	/* Don't print wrapped token bytestring */
    print_claim(&cose_sign1_wrapper[2], 0);
    println!("== End of {} Token cose header\n", token_type);
}

#[allow(dead_code)]
pub fn print_token_rust(claims: &bindgen::attestation_claims)
{
    print_cose_sign1_wrapper("Realm", &claims.realm_cose_sign1_wrapper);

    println!("== Realm Token:");
    for token in &claims.realm_token_claims {
        print_claim(token, 0);
    }
    println!("{:COLUMN$} (#{})", "Realm measurements", bindgen::CCA_REALM_EXTENSIBLE_MEASUREMENTS);
    for claim in &claims.realm_measurement_claims {
        print_claim(claim, 1);
    }
    println!("== End of Realm Token.\n\n");

    print_cose_sign1_wrapper("Platform", &claims.plat_cose_sign1_wrapper);

    println!("== Platform Token:");
    for claim in &claims.plat_token_claims {
        print_claim(claim, 0);
    }
    println!("== End of Platform Token\n");

    let mut count = 0;
    println!("== Platform Token SW components:");
    for component in &claims.sw_component_claims {
        if component.present {
            println!("  SW component #{}:", count);
            for claim in &component.claims {
                print_claim(&claim, 2)
            }
            count += 1;
        }
    }
	println!("== End of Platform Token SW components\n\n");
}
