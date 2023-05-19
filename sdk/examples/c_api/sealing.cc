#include "../../include/islet.h"

#include <iostream>
#include <string>
#include <cstring>

int main() {
  using byte = unsigned char;

  byte sealed[2048], unsealed[2048];
  memset(sealed, 0, sizeof(sealed));
  memset(unsealed, 0, sizeof(unsealed));
  int sealed_len = 0, unsealed_len = 0;

  // -- Seal -- //
  std::string plaintext("Plaintext");
  if (islet_seal((const byte*)plaintext.c_str(), plaintext.size(), sealed, &sealed_len))
    return -1;

  // -- Unseal -- //
  if (islet_unseal(sealed, sealed_len, unsealed, &unsealed_len))
    return -1;

  printf("Success sealing round trip.\n");

  return 0;
}
