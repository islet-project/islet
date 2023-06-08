# ISLET
ISLET is a project to enable on-device confidential computing
for end users by leveraging ARMv9 CCA that is
the newly emerging confidential computing hardware on ARM devices.
Using the hardware support, ISLET enables a Trusted Execution Environment (TEE)
on user’s devices within which users can securely process, store, communicate
and manage their private data. The protection provided by
ISLET applies not only to data-at-rest but also to data-in-use
even in the presence of malicious privileged software on devices.

We develop components enabling Realm Virtual Machines (VMs),
which are secure VM-level TEE provided by ARMv9 CCA.
To manage Realm VMs, Realm Management Monitor (RMM)
is needed to be running at EL2 in the Realm world.
ISLET provides the implementation of RMM that is written in Rust. 

## Software components
- [Host Linux](https://gitlab.arm.com/linux-arm/linux-cca): Linux supported Arm CCA running on normal world
- Islet RMM: Rust-based Realm Management Monitor running on realm world EL2
- [TF RMM](https://www.trustedfirmware.org/projects/tf-rmm/): C-based Realm Management Monitor running on realm world EL2
- [Linux Realm](https://gitlab.arm.com/linux-arm/linux-cca): Linux running on realm world EL0-1
- WASM Realm: Linux with root filesystem built `wasmer` running on realm world EL0-1

## Command Line Interface
ISLET provides Command Line Interface(CLI) which can explore CCA operations.
CLI supports both x86_64(simulated) and aarch64.

You can explore Attestation like below
```sh
$ cd cli
$ make x86_64
$ ./islet attest --output=./report
$ ./islet verify --input=./report

== Signature Verification:
Sign Algo        = [ES384]
Public Key       = ["0476f988091be585ed41801aecfab858...]
Data             = ["846a5369676e61747572653144a10138...]
Signature        = ["ec4f3b28a00feabd1f58f94acb27fdc7...]
== End of Signature Verification

== Realm Token cose:
Protected header               = Header { alg: Some(Assigned(ES
Unprotected header             = Header { alg: None, crit: [],
Signature                      = [ec4f3b28a00feabd1f58f94acb27f
== End of Realm Token cose

== Realm Token:
Realm challenge                (#10) = [abababababababababababa
Realm personalization value    (#44235) = [00000000000000000000
Realm hash algo id             (#44236) = "sha-256"
Realm public key hash algo id  (#44240) = "sha-256"
Realm signing public key       (#44237) = [0476f988091be585ed41
Realm initial measurement      (#44238) = [6190eb90b293886c172e
```

## Getting started on FVP
### Installing dependencies
```bash
./scripts/init.sh
```

### Running a linux realm
```bash
// Start FVP on host
$ ./scripts/fvp-cca --normal-world=linux --realm=linux --rmm=tf-rmm

// Run a linux realm on fvp
$ ./launch-realm.sh
```

### Running SDK sample apps after [running a linux realm](#running-a-linux-realm)
```bash
// Move to shared dir on realm
$ cd /shared

// Insert RSI kernel module
$ inmod rsi.ko

// Run the sample app (rust)
$ ./sdk-example

// Run the sample app (c)
$ LD_LIBRARY_PATH=./ ./sdk-example-c
```

### Running a linux realm with a networking support and prebuilt examples
See [examples](./examples/README.md).
To get details about its network configuration, see [network.md](./docs/network.md).

### Testing the realm features
```bash
// Start FVP on fvp
$ ./scripts/fvp-cca --normal-world=linux --realm=linux --rmm=tf-rmm

// Test the realm features on fvp
$ ./test-realm.sh [attest]
```

### Testing RMMs with tf-a-tests
```
# Islet RMM
$ ./scripts/fvp-cca --normal-world=tf-a-tests --rmm=islet

# TF RMM
$ ./scripts/fvp-cca --normal-world=tf-a-tests --rmm=tf-rmm
```

### Testing RMMs with [ACS](https://github.com/ARM-software/cca-rmm-acs)
```
# TF RMM
$ ./scripts/fvp-cca --normal-world=acs --rmm=tf-rmm
```
