#include "../../include/islet.h"

#include <iostream>
#include <string>
#include <cstring>

static const char CLAIM_TITLE_USER_DATA[] = "User data";
static const char CLAIM_TITLE_PLATFORM_PROFILE[] = "Profile";

int main() {
  using byte = unsigned char;

  byte report[2048], claims[1024], value[1024];
  memset(report, 0, sizeof(report));
  memset(claims, 0, sizeof(report));
  memset(value, 0, sizeof(value));
  int report_len = 0, claims_len = 0, value_len = 0;

  // -- Attest -- //
  std::string user_data("User Custom data");
  if (islet_attest((const byte*)user_data.c_str(), user_data.size(), report, &report_len))
    return -1;

  // -- Verify -- //
  if (islet_verify(report, report_len, claims, &claims_len))
    return -1;

  islet_print_claims(claims, claims_len);

  // -- Parse -- //
  if (islet_parse(CLAIM_TITLE_USER_DATA, claims, claims_len, value, &value_len))
    return -1;

  printf("Claim[User data]: %s\n", (char*) value);

  memset(value, 0, sizeof(value));
  if (islet_parse(CLAIM_TITLE_PLATFORM_PROFILE, claims, claims_len, value, &value_len))
    return -1;

  printf("Claim[Platform  profile]: %s\n", (char*) value);

  return 0;
}
