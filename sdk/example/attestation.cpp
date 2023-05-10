#include <islet.h>

#include <iostream>
#include <string>
#include <vector>

int main() {
    using byte = unsigned char;

    // -- Attest -- //
    std::vector<byte> report_out(2048);
    int report_out_len;

    std::string user_data("User custom data");
    if (islet_attest((const unsigned char*)user_data.c_str(),
                     user_data.size(),
                     report_out.data(),
                     &report_out_len) != 0) {
        return -1;
    }
    std::cout << "Success to get an attestation report." << std::endl;

    // -- Verify -- //
    std::vector<byte> claims_out(2048);
    int claims_out_len;
    if (islet_verify(report_out.data(), report_out_len, claims_out.data(), &claims_out_len) != 0) {
        return -1;
    }
    std::cout << "Success to verify the attestation report and get attestation claims." << std::endl;

    // -- Debug print -- //
    islet_print_claims(claims_out.data(), claims_out_len);

    // -- Get claim value -- //
    std::vector<byte> value_out(2048);
    int value_out_len;
    if (islet_parse("User data", claims_out.data(), claims_out_len, value_out.data(), &value_out_len)) {
        return -1;
    }
    std::cout << "Claim-User data: " << std::string(value_out.begin(), value_out.end()) << std::endl;

    if (islet_parse("Profile", claims_out.data(), claims_out_len, value_out.data(), &value_out_len)) {
        return -1;
    }
    std::cout << "Claim-Platform profile: " << std::string(value_out.begin(), value_out.end()) << std::endl;

    return 0;
}
