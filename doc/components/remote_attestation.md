# Introduction

Remote Attestation (RA) is the key of confidential computing platform, which is
basically a method that convinces to *verifier* that a program (*attester*) is
running on a proper confidential computing platform (e.g., ARM CCA).

## Report

An attestation *report* is an *evidence* produced by *attester* and consumed by
*verifier*. In ARM CCA, report consists of two different tokens:

- *CCA Platform token*: it is used to assure that *attester* is running on a
  secure CCA platform. It covers the measurements of CCA platform components
  (e.g., RMM and EL3) and whether it is in debug state.
- *Realm token*: this token is used to hold the measurements of Realm, which is
  equivalent to a virtual machine that may contain kernel and root file system.

You can test and see what this report (in form of CCA token) looks like through
[our CLI tool](https://github.com/islet-project/rsictl) and the [Veraison
attestation
demo](https://github.com/islet-project/islet/blob/main/examples/veraison/RUN.md).

## Delegated Attestation in Islet RMM

The implementation in Islet RMM conforms to [Realm Management Monitor
specification version
1.0-REL0](https://developer.arm.com/documentation/den0137/1-0rel0/?lang=en).

This section describes how delegated attestation works in Islet RMM implementation.
The process involves two tokens that together form the complete attestation evidence.

### Two Tokens

As mentioned before, the attestation report consists of two tokens:

- **CCA Platform Token**: This token is signed by the CCA Platform Attestation
  Key (CPAK). It contains measurements of platform components like bootloaders,
  RMM firmware, and EL3 monitor.
- **Realm Token**: This token is signed by the Realm Attestation Key (RAK). It
  contains measurements specific to the Realm (virtual machine).

### RAK and Platform Token from HES

The RAK key pair and the CCA platform token are fetched from HES (Hardware
Enforced Security). When RMM initializes, it requests these from TF-A, which
then forwards the request to HES.

The process happens in
[`rmm/src/rmm_el3/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rmm_el3/mod.rs)
during the `setup_el3_ifc()` function:

1. `get_realm_attest_key()` - fetches the RAK private key
2. `get_plat_token()` - fetches the CCA platform token

The actual SMC calls to TF-A are implemented in
[`rmm/src/rmm_el3/iface.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rmm_el3/iface.rs).

### Binding Between Tokens

The RAK public key (and therefore the Realm token) is bound to the CCA
platform token. This binding is done through the challenge field in the
platform token.

The challenge is computed as `hash(RAK_pub)`, where RAK_pub is the public part
of the Realm Attestation Key. This hash computation happens in
[`rmm/src/rmm_el3/digest.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rmm_el3/digest.rs).

When the platform token is requested, RMM sends the hash of RAK public key to
HES. HES includes this hash as the challenge claim in the platform token. This
creates a cryptographic binding between the two tokens.

### Caching

The RAK private key and the platform token are cached by RMM after the initial
fetch. They are stored in static variables protected by spinlocks in
[`rmm/src/rmm_el3/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rmm_el3/mod.rs):

- `REALM_ATTEST_KEY` - stores the RAK private key
- `PLAT_TOKEN` - stores the CCA platform token

This caching is important because:
- The RAK private key is used later to sign Realm attestation tokens
- The platform token is included in every complete attestation response
- Fetching from HES each time would be slow and unnecessary

### Token Format

Both tokens use COSE (CBOR Object Signing and Encryption) format.
Specifically, they are wrapped in COSE_Sign1 structure, which means a single signature.

The format follows the [RMM spec. 1.0-REL0 A7.2.1](https://developer.arm.com/documentation/den0137/1-0rel0).

The complete CCA attestation token is a CBOR map that contains:

```
cca-token = #6.399(cca-token-collection) ; CMW Collection
                                         ; (draft-ietf-rats-msg-wrap)
cca-platform-token = bstr .cbor COSE_Sign1_Tagged
cca-realm-delegated-token = bstr .cbor COSE_Sign1_Tagged
cca-token-collection = {
    44234 => cca-platform-token          ; 44234 = 0xACCA
    44241 => cca-realm-delegated-token
}

; EAT standard definitions
COSE_Sign1_Tagged = #6.18(COSE_Sign1)

; Deliberately shortcut these definitions until EAT is finalised and able to
; pull in the full set of definitions
COSE_Sign1 = "COSE-Sign1 placeholder"
```

The signing algorithm used is ES384 (ECDSA with SHA-384 and P-384 curve).  You
can see the token construction in
[`rmm/src/rsi/attestation/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rsi/attestation/mod.rs).

## Measurement Extend RSI Handling

Realm measurements are stored in measurement slots. There are 5 slots in total:

- Slot 0: RIM (Realm Initial Measurement)
- Slots 1-4: REMs (Realm Extensible Measurements)

The measurements are described in [RMM spec. 1.0-REL0 A7.1](https://developer.arm.com/documentation/den0137/1-0rel0).

The measurement slot definitions are in
[`rmm/src/measurement/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/measurement/mod.rs).

### RSI_MEASUREMENT_EXTEND

This RSI call is defined in [RMM spec. 1.0-REL0
B5.3.7](https://developer.arm.com/documentation/den0137/1-0rel0).

When a Realm calls `RSI_MEASUREMENT_EXTEND`, RMM extends one of the extensible
measurement slots. The handler is implemented in
[`rmm/src/rsi/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rsi/mod.rs).

The extend operation works like this:

1. Read the current measurement value from the specified slot
2. Hash together: old measurement value + new data
3. Store the result back to the measurement slot

The actual hash computation is done in
[`rmm/src/measurement/ctx.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/measurement/ctx.rs)
in the `extend_measurement()` function.

Note that slot 0 (RIM) cannot be extended via RSI. It is set during Realm
creation and REC (Realm Execution Context) creation.

### Measurement Storage

Measurements are stored in the Realm Descriptor (Rd) structure. The Rd
structure is defined in
[`rmm/src/realm/rd.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/realm/rd.rs).

Each measurement slot can hold up to 64 bytes (512 bits), which supports SHA-512
hashes.

## Attestation Token Retrieval RSI Handling

The attestation token retrieval uses two RSI calls: INIT and CONTINUE. This is
because the complete token can be large and may not fit in registers.

### RSI_ATTEST_TOKEN_INIT

This RSI call is defined in [RMM spec. 1.0-REL0
B5.3.2](https://developer.arm.com/documentation/den0137/1-0rel0).

This call initializes the attestation process. The handler is in
[`rmm/src/rsi/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rsi/mod.rs).

The caller provides a 64-byte challenge (nonce) in registers x1-x8. RMM stores
this challenge and prepares for token generation.

The response includes the total token size, so the caller knows how much to
allocate.

### RSI_ATTEST_TOKEN_CONTINUE

This RSI call is defined in [RMM spec. 1.0-REL0
B5.3.1](https://developer.arm.com/documentation/den0137/1-0rel0).

This call retrieves the token in chunks. The handler is in
[`rmm/src/rsi/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rsi/mod.rs).

The caller provides:
- IPA (Intermediate Physical Address) where to write the token
- Offset within the token
- Buffer size

RMM returns a portion of the token and indicates if more data is available.

### Token Construction

The token is constructed in
[`rmm/src/rsi/attestation/mod.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rsi/attestation/mod.rs)
in the `get_token()` function:

1. Create Realm claims with:
   - Challenge (from caller)
   - RIM and REMs (from Rd)
   - Personalization value (from Rd)
   - RAK public key

2. Build the Realm token as CBOR map and sign it with RAK private key

3. Combine with cached platform token into the complete CCA attestation token

The Realm claims structure is defined in
[`rmm/src/rsi/attestation/claims.rs`](https://github.com/islet-project/islet/tree/main/rmm/src/rsi/attestation/claims.rs).

The final token is returned to the Realm, which can then send it to a verifier
for attestation.
