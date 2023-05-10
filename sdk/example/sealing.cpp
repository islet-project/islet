#include "../include/islet.h"

#include <iostream>
#include <string>
#include <vector>

int main() {
    using byte = unsigned char;

    // -- Seal -- //
    std::vector<byte> sealed_out(2048);
    int sealed_out_len;

    std::string plaintext("Plaintext");
    if (islet_seal((const unsigned char*)plaintext.c_str(),
                     plaintext.size(),
                     sealed_out.data(),
                     &sealed_out_len) != 0) {
        return -1;
    }
    std::cout << "Success to seal the plaintext. sealed_out_len: "
              << sealed_out_len << std::endl;

    // -- Unseal -- //
    std::vector<byte> plaintext_out(2048);
    int plaintext_out_len;

    if (islet_unseal(sealed_out.data(),
                     sealed_out_len,
                     plaintext_out.data(),
                     &plaintext_out_len) != 0) {
        return -1;
    }
    std::cout << "Success to unseal the sealed: "
              << std::string(plaintext_out.begin(), plaintext_out.end()) << std::endl;


    return 0;
}
