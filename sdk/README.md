# Islet SDK
Islet SDK is an open source SDK
which provides confidential computing primitives to enclaves.
There are two types of component which can use our API.
One is the application type running on EL0.
The other is the VM type running on EL1.
Islet SDK focuses the application type first.
We believe that you can easily migrate
the existing applications to the Arm CCA World.

## Supported Languages
Islet SDK is written in `rust` but we also support `C/C++`.
We use `cbindgen` which is the powerful tool
to create headers for rust libraries which expose public C/C++ APIs.

```
+--------+                           +---------+
| sdk    | => cbindgen => header  => | app     |
| (rust) | => cdylib   => library => | (c/c++) |
+--------+                           +---------+
```

## Confidential Computing Primitives
Currently Islet SDK provides `Attestation` and `Sealing`. You can check reference code snippets.

### Attestation
#### Rust code snippet
```rust
use islet_sdk::prelude::*;

let user_data = b"User data";
let report = attest(user_data)?;
let claims = verify(&report)?;

print_claim(&claims);

if let Some(ClaimData::Bstr(data)) = parse(&claims, config::STR_USER_DATA) {
    assert_eq!(user_data, &data[..user_data.len()]);
} else {
    assert!(false, "Wrong user data");
}

if let Some(ClaimData::Text(data)) = parse(&claims, config::STR_PLAT_PROFILE) {
    assert_eq!(data, "http://arm.com/CCA-SSD/1.0.0");
} else {
    assert!(false, "Wrong platform profile");
}

if let Some(ClaimData::Bstr(data)) = parse(&claims, config::STR_REALM_INITIAL_MEASUREMENT) {
    println!("Realm initial measurement: {:X?}", &data);
} else {
    assert!(false, "Wrong RIM");
}
```

#### C++ code snippet
```cpp
using byte = unsigned char;

byte report[2048], claims[1024], claim[1024];
int report_len, claims_len, claim_len;

std::string user_data("User Custom data");
if (!islet_attest((const byte*)user_data.c_str(), user_data.size(), report, &report_len))
    return -1;

if (!islet_verify(report, report_len, claims, &claims_len))
    return -1;

islet_print_claims(claims, claims_len);

const char CLAIM_USER_DATA[] = "User data"; // Claim title 
if (!islet_parse("User data", claims_out.data(), claims_out_len, value_out.data(), &value_out_len))
    return -1;

printf("Claim[User data]: %s\n", (char*) value);
```

### Sealing
#### Rust code snippet
```rust
let plaintext = b"Plaintext";
let sealed = seal(plaintext)?;
let unsealed = unseal(&sealed)?;
assert_eq!(plaintext, &unsealed[..]);   
```

#### C++ code snippet
```cpp
using byte = unsigned char;

byte sealed[2048], unsealed[2048];
memset(sealed, 0, sizeof(sealed));
memset(unsealed, 0, sizeof(unsealed));
int sealed_len = 0, unsealed_len = 0;

std::string plaintext("Plaintext");
if (islet_seal((const byte*)plaintext.c_str(), plaintext.size(), sealed, &sealed_len))
    return -1;

if (islet_unseal(sealed, sealed_len, unsealed, &unsealed_len))
    return -1;

printf("Success sealing round trip.\n");
```
