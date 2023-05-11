// Copyright (c) 2023 Samsung Electronics Co., Ltd. All Rights Reserved.

#pragma once

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

constexpr static const int ISLET_E_SUCCESS = 0;

constexpr static const int ISLET_E_FAILURE = -1;

constexpr static const int ISLET_E_INPUT = -2;

constexpr static const int ISLET_E_WRONG_REPORT = -3;

constexpr static const int ISLET_E_WRONG_CLAIMS = -4;

constexpr static const int ISLET_E_NOT_SUPPORTED = -5;

extern "C" {

/// Get an attestation report(token).
///
/// # Note
/// This API returns hard coded report to simuate temporarily.
/// `User data` could be used as nonce to prevent reply attack.
int islet_attest(const unsigned char *user_data,
                 int user_data_len,
                 unsigned char *report_out,
                 int *report_out_len);

/// Verify the attestation report and returns attestation claims if succeeded.
int islet_verify(const unsigned char *report,
                 int report_len,
                 unsigned char *claims_out,
                 int *claims_out_len);

/// Parse the claims with the given title and returns the claim if succeeded.
int islet_parse(const char *title,
                const unsigned char *claims,
                int claims_len,
                unsigned char *value_out,
                int *value_out_len);

/// Print all claims including Realm Token and Platform Token.
void islet_print_claims(const unsigned char *claims, int claims_len);

/// Seals the plaintext given into the binary slice
///
/// # Note
/// This API seals with hard coded key to simuate temporarily.
int islet_seal(const unsigned char *plaintext,
               int plaintext_len,
               unsigned char *sealed_out,
               int *sealed_out_len);

/// Unseals the sealed bianry given into the plaintext
///
/// # Note
/// This API unseals with hard coded key to simuate temporarily.
int islet_unseal(const unsigned char *sealed,
                 int sealed_len,
                 unsigned char *plaintext_out,
                 int *plaintext_out_len);

} // extern "C"
