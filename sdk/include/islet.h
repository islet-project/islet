/*
 *  Copyright (c) 2023 Samsung Electronics Co., Ltd All Rights Reserved
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the language governing permissions and
 *  limitations under the License
 */

#pragma once

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum islet_status_t {
  ISLET_SUCCESS = 0,
  ISLET_FAILURE = -1,
  ISLET_ERROR_INPUT = -2,
  ISLET_ERROR_WRONG_REPORT = -3,
  ISLET_ERROR_WRONG_CLAIMS = -4,
  ISLET_ERROR_FEATURE_NOT_SUPPORTED = -5,
};

extern "C" {

/// Get an attestation report(token).
///
/// # Note
/// This API currently returns hard-coded report to simulate attest operation.
/// In future, this will be finalized to support reports signed by RMM.
/// `User data` could be used as nonce to prevent reply attack.
islet_status_t islet_attest(const unsigned char *user_data,
                            int user_data_len,
                            unsigned char *report_out,
                            int *report_out_len);

/// Verify the attestation report and returns attestation claims if succeeded.
islet_status_t islet_verify(const unsigned char *report,
                            int report_len,
                            unsigned char *claims_out,
                            int *claims_out_len);

/// Parse the claims with the given title and returns the claim if succeeded.
islet_status_t islet_parse(const char *title,
                           const unsigned char *claims,
                           int claims_len,
                           unsigned char *value_out,
                           int *value_out_len);

/// Print all claims including Realm Token and Platform Token.
void islet_print_claims(const unsigned char *claims, int claims_len);

/// Seals the plaintext given into the binary slice
///
/// # Note
/// This API currently seals with a hard-coded key, to simulate seal operation.
/// In future, this will be finalized to support keys derived from HES.
islet_status_t islet_seal(const unsigned char *plaintext,
                          int plaintext_len,
                          unsigned char *sealed_out,
                          int *sealed_out_len);

/// Unseals into plaintext the sealed binary provided.
///
/// # Note
/// This API currently unseals with a hard-coded key, to simulate unseal operation.
/// In future, this will be finalized to support keys derived from HES.
islet_status_t islet_unseal(const unsigned char *sealed,
                            int sealed_len,
                            unsigned char *plaintext_out,
                            int *plaintext_out_len);

} // extern "C"
