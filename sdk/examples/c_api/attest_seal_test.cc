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

#include <islet.h>

#include <iostream>
#include <string>
#include <cstring>

#define BUFFER_SIZE 2048

using byte = unsigned char;

static const char CLAIM_TITLE_USER_DATA[] = "User data";
static const char CLAIM_TITLE_PLATFORM_PROFILE[] = "Profile";

bool attestation_test() {
  byte report[BUFFER_SIZE];
  byte claims[BUFFER_SIZE];
  byte value[BUFFER_SIZE];
  int report_len = 0;
  int claims_len = 0;
  int value_len = 0;

  memset(report, 0, sizeof(report));
  memset(claims, 0, sizeof(report));
  memset(value, 0, sizeof(value));

  // -- Attest -- //
  std::string user_data("User Custom data");
  if (islet_attest((const byte*)user_data.c_str(), user_data.size(), report, &report_len))
    return false;

  // -- Verify -- //
  if (islet_verify(report, report_len, claims, &claims_len))
    return false;

  islet_print_claims(claims, claims_len);

  // -- Parse -- //
  if (islet_parse(CLAIM_TITLE_USER_DATA, claims, claims_len, value, &value_len))
    return false;

  printf("Claim[User data]: %s\n", (char*) value);

  memset(value, 0, sizeof(value));
  if (islet_parse(CLAIM_TITLE_PLATFORM_PROFILE, claims, claims_len, value, &value_len))
    return false;

  printf("Claim[Platform  profile]: %s\n", (char*) value);

  return true;
}

bool sealing_test() {
  byte sealed[BUFFER_SIZE];
  byte unsealed[BUFFER_SIZE];

  int sealed_len = 0;
  int unsealed_len = 0;

  memset(sealed, 0, sizeof(sealed));
  memset(unsealed, 0, sizeof(unsealed));

  // -- Seal -- //
  std::string plaintext("Plaintext");
  if (islet_seal((const byte*)plaintext.c_str(), plaintext.size(), sealed, &sealed_len))
    return false;

  // -- Unseal -- //
  if (islet_unseal(sealed, sealed_len, unsealed, &unsealed_len))
    return false;

  printf("Success sealing round trip.\n");

  return true;
}

int main() {
  bool rv = attestation_test();
  printf("Attestation test %s.\n", (rv ? "succeeded" : "failed"));
  if (!rv)
    return -1;

  rv = sealing_test();
  printf("Sealing test %s.\n", (rv ? "succeeded" : "failed"));

  return 0;
}
