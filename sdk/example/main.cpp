#include <iostream>
#include "../islet.h"

int main() {
    // -- Attest -- //
    unsigned char report_out[2048];
    int report_out_len;
    
    if (islet_attest((uint8_t*) "123", 3, report_out, &report_out_len) != 0) {
        return -1;
    }

    std::cout << "Success to get an attestation report" << std::endl;

    // -- Verify -- //
    unsigned char claims_out[2048];
    int claims_out_len;
    if (islet_verify(report_out, report_out_len, claims_out, &claims_out_len) != 0) {
        return -1;
    }

    std::cout << "Success to verify the attestation report and get attestation claims." << std::endl;

    // -- Get claim value -- //
    uint8_t value_out[2048];
    int32_t value_out_len;
    if (islet_parse("User data", claims_out, claims_out_len, value_out, &value_out_len)) {
        return -1;
    }
    std::cout << "Claim-User data: " << reinterpret_cast<char *>(value_out) << std::endl;

    if (islet_parse("Profile", claims_out, claims_out_len, value_out, &value_out_len)) {
        return -1;
    }
    std::cout << "Claim-Platform profile: " << reinterpret_cast<char *>(value_out) << std::endl;

    return 0;
}
