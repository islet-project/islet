# Islet software dependencies

## The main Islet repository

* Repository: https://github.com/islet-project/islet
* Origin: written by us
* License: Apache-2.0

This is the main repository of the Islet project. The repository contains implemetation of `Islet RMM` (Realm Management Monitor), `Islet HES` (Hardware Enforced Security) and Islet SDK. It contains also scripts that allows to setup, build, test and run the whole Arm CCA stack on FVP and Qemu emulators. Additionally it provides comprehensive documentation and examples.

### Islet RMM dependencies

| Name                  | Build time         | Runtime            | License                   | URL                                                            |
| --------------------- | :----------------: | :----------------: | ------------------------- | -------------------------------------------------------------- |
| aarch64-cpu           | :heavy_check_mark: | :heavy_check_mark: | MIT/Apache-2.0            | https://github.com/rust-embedded/aarch64-cpu                   |
| bitflags              | :heavy_check_mark: | :heavy_check_mark: | MIT/Apache-2.0            | https://github.com/bitflags/bitflags                           |
| cc                    | :heavy_check_mark: |                    | MIT OR Apache-2.0         | https://github.com/rust-lang/cc-rs                             |
| ciborium              | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0                | https://github.com/enarx/ciborium                              |
| coset                 | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0                | https://github.com/google/coset                                |
| ecdsa                 | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT         | https://github.com/RustCrypto/signatures/tree/master/ecdsa     |
| hex                   | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/KokaKiwi/rust-hex                           |
| hkdf                  | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/RustCrypto/KDFs/                            |
| lazy_static           | :heavy_check_mark: | :heavy_check_mark: | MIT/Apache-2.0            | https://github.com/rust-lang-nursery/lazy-static.rs            |
| libfuzzer-sys         | :heavy_check_mark: | :heavy_check_mark: | (MIT OR Apache-2.0) AND NCSA | https://github.com/rust-fuzz/libfuzzer                      |
| linked_list_allocator | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0/MIT            | https://github.com/phil-opp/linked-list-allocator              |
| lock_api              | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/Amanieu/parking_lot                         |
| log                   | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/rust-lang/log                               |
| p384                  | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT         | https://github.com/RustCrypto/elliptic-curves/tree/master/p384 |
| paste                 | :heavy_check_mark: |                    | MIT OR Apache-2.0         | https://github.com/dtolnay/paste                               |
| sha2                  | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/RustCrypto/hashes                           |
| spin                  | :heavy_check_mark: | :heavy_check_mark: | MIT                       | https://github.com/mvdnes/spin-rs.git                          |
| spinning_top          | :heavy_check_mark: | :heavy_check_mark: | MIT/Apache-2.0            | https://github.com/rust-osdev/spinning_top                     |
| tinyvec               | :heavy_check_mark: | :heavy_check_mark: | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec                            |
| tock-registers        | :heavy_check_mark: | :heavy_check_mark: | MIT/Apache-2.0            | https://github.com/tock/tock/tree/master/libraries/tock-register-interface |


### Islet SDK dependencies
| Name     | Build time         | Runtime            | License           | URL                                                            |
| -------- | :----------------: | :----------------: | ----------------- | -------------------------------------------------------------- |
| bincode  | :heavy_check_mark: | :heavy_check_mark: | MIT               | https://github.com/servo/bincode                               |
| bindgen  | :heavy_check_mark: |                    | BSD-3-Clause      | https://github.com/rust-lang/rust-bindgen                      |
| cc       | :heavy_check_mark: |                    | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs                             |
| cbindgen | :heavy_check_mark: |                    | MPL-2.0           | https://github.com/eqrion/cbindgen                             |
| cfg_if   | :heavy_check_mark: |                    | MIT/Apache-2.0    | https://github.com/alexcrichton/cfg-if                         |
| ciborium | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0        | https://github.com/enarx/ciborium                              |
| coset    | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0        | https://github.com/google/coset                                |
| ecdsa    | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/signatures/tree/master/ecdsa     |
| hex      | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex                           |
| nix      | :heavy_check_mark: | :heavy_check_mark: | MIT               | https://github.com/nix-rust/nix                                |
| openssl  | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0        | https://github.com/sfackler/rust-openssl                       |
| p256     | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/elliptic-curves/tree/master/p256 |
| p384     | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/elliptic-curves/tree/master/p384 |
| p521     | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/elliptic-curves/tree/master/p521 |
| serde    | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/serde-rs/serde                              |
| rust-rsi (our own library — see [rust-rsi](#rust-rsi)) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/rust-rsi                        |
| sha2     | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes                           |
| zeroize  | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils                            |


### Islet HES dependencies
| Name           | Build time         | Runtime            | License                   | URL                                                             |
| -------------- | :----------------: | :----------------: | ------------------------- | ----------------------------------------------------------------|
| aes            | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/RustCrypto/block-ciphers                     |
| ciborium       | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0                | https://github.com/enarx/ciborium                               |
| clap           | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/clap-rs/clap                                 |
| clean-path     | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://gitlab.com/foo-jin/clean-path                           |
| coset          | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0                | https://github.com/google/coset                                 |
| daemonize      | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/knsd/daemonize                               |
| ecdsa          | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT         | https://github.com/RustCrypto/signatures/tree/master/ecdsa      |
| elliptic-curve | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT         | https://github.com/RustCrypto/traits/tree/master/elliptic-curve |
| hex            | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/KokaKiwi/rust-hex                            |
| hkdf           | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/RustCrypto/KDFs/                             |
| p256           | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT         | https://github.com/RustCrypto/elliptic-curves/tree/master/p256  |
| p384           | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT         | https://github.com/RustCrypto/elliptic-curves/tree/master/p384  |
| p521           | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT         | https://github.com/RustCrypto/elliptic-curves/tree/master/p521  |
| sha2           | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0         | https://github.com/RustCrypto/hashes                            |
| tinyvec        | :heavy_check_mark: | :heavy_check_mark: | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec                             |

## Submodules of islet (third-party directories in the main islet repository)

The main islet repository uses external dependencies located in third-party and assets folder. Here is the list of this dependencies.

| Submodule | Repository | Tracked ref | License | Used for |
| --------- | ---------- | ----------- | ------- | -------- |
| [`assets`](#assets-submodule) | [assets](https://github.com/islet-project/assets) | `main` | mixed — prebuilt third-party content | pinned Arm GNU toolchains, `dtc` with a prebuilt `libfdt.a`, EDK2 firmware images, rootfs |
| [`third-party/nw-linux`](#third-partynw-linux) | [3rd-linux](https://github.com/islet-project/3rd-linux) | `cca-host/v5` | GPL-2.0-only | normal-world (host) kernel of the FVP and QEMU runs |
| [`third-party/realm-linux`](#third-partyrealm-linux) | [3rd-linux](https://github.com/islet-project/3rd-linux) | `cca-guest/v6` | GPL-2.0-only | realm (guest) kernel; kbuild tree of `rsi.ko` |
| [`third-party/android-kernel`](#third-partyandroid-kernel) | [3rd-android-kernel](https://github.com/islet-project/3rd-android-kernel) | `main` | GPL-2.0-only | Android CC host and Microdroid guest kernels |
| [`third-party/tf-a`](#third-partytf-a) | [3rd-tf-a](https://github.com/islet-project/3rd-tf-a) | `main` | BSD-3-Clause | EL3/EL1 boot firmware of the FVP and QEMU images |
| [`third-party/tf-a-rss`](#third-partytf-a-rss) | [3rd-tf-a](https://github.com/islet-project/3rd-tf-a) | `hes` | BSD-3-Clause | the same repository, TF-A + RSS firmware used with `--hes` |
| [`third-party/tf-a-tests`](#third-partytf-a-tests) | [3rd-tf-a-tests](https://github.com/islet-project/3rd-tf-a-tests) | `main` | BSD-3-Clause | TFTF test payload run as the normal world |
| [`third-party/tf-rmm`](#third-partytf-rmm) | [3rd-tf-rmm](https://github.com/islet-project/3rd-tf-rmm) | `main` | BSD-3-Clause | reference RMM — alternative monitor and SDK reference |
| [`third-party/cca-rmm-acs`](#third-partycca-rmm-acs) | [cca-rmm-acs](https://github.com/islet-project/cca-rmm-acs) | `islet-v1.0-rel0` | BSD-3-Clause | Arm CCA RMM compliance (ACS) test suite |
| [`third-party/kvmtool`](#third-partykvmtool) | [3rd-kvmtool](https://github.com/islet-project/3rd-kvmtool) | `cca/v3` | GPL-2.0-only | `lkvm`, the CCA-capable host VMM that boots realms |
| [`third-party/kvmtool-rim-measurer`](#third-partykvmtool-rim-measurer) | [3rd-kvmtool](https://github.com/islet-project/3rd-kvmtool) | `kvmtool-rim-measurer/eac5` | GPL-2.0-only | `lkvm-rim-measurer` — realm initial measurement and metadata |
| [`third-party/kvm-unit-tests`](#third-partykvm-unit-tests) | [kvm-unit-tests-cca](https://git.gitlab.arm.com/linux-arm/kvm-unit-tests-cca.git) | `cca/rmm-v1.0-eac5` | GPL-2.0-only | realm unit tests run under `lkvm` |
| [`third-party/qemu`](#third-partyqemu) | [qemu](https://github.com/qemu/qemu.git) | `stable-9.2`, pinned `v9.2.4` | GPL-2.0-only (+ LGPL-2.1 utilities) | Arm CCA/RME-capable emulator — see the note below |
| [`third-party/mbedtls`](#third-partymbedtls) | [mbedtls](https://github.com/Mbed-TLS/mbedtls.git) | `v3.6.4` | Apache-2.0 OR GPL-2.0-or-later | crypto library of the TF-A RSS build |
| [`third-party/dtc`](#third-partydtc) | [dtc](https://git.kernel.org/pub/scm/utils/dtc/dtc.git) | pinned `18aa49a` | GPL-2.0-or-later OR BSD-2-Clause | device tree compiler of the provisioning example |
| [`third-party/certifier`](#third-partycertifier) | [certifier-framework](https://github.com/ccc-certifier-framework/certifier-framework-for-confidential-computing) | pinned SHA | Apache-2.0 | Certifier Service/API used by our examples |
| [`third-party/ciborium`](#third-partyciborium) | [ciborium](https://github.com/islet-project/ciborium) | `3rd-ciborium` | Apache-2.0 | CBOR path dependency of `rmm`, `hes` and `sdk` |
| [`third-party/coset`](#third-partycoset) | [coset](https://github.com/islet-project/coset) | `3rd-coset` | Apache-2.0 | COSE path dependency of `rmm`, `hes` and `sdk` |
| [`third-party/cargo-geiger`](#third-partycargo-geiger) | [cargo-geiger](https://github.com/bitboom/cargo-geiger) | pinned SHA (`master`) | Apache-2.0 OR MIT (no root license file) | development-only unsafe-usage analysis |
| [`third-party/linux-rsi`](#third-partylinux-rsi) | [linux-rsi](https://github.com/islet-project/linux-rsi) | pinned SHA | GPL-2.0-only | the `rsi.ko` module — described in its [own section](#linux-rsi) |
| [`third-party/optee-build`](#third-partyoptee-build) | [assets](https://github.com/islet-project/assets) | `3rd-optee-build-cca-v5` | BSD-2-Clause / BSD-3-Clause per file | OP-TEE build system that assembles the FVP images |

### third-party/nw-linux

* Repository: https://github.com/islet-project/3rd-linux, branch `cca-host/v5`
* Upstream: the Linux kernel — forked so that the host-side CCA changes live in our organization
* License: **GPL-2.0-only**, `WITH Linux-syscall-note` on the UAPI headers (`COPYING` + `LICENSES/`)

The normal-world (host) kernel booted by the FVP and QEMU Islet configurations: it carries the RMI host-side driver and the Realm management patches that the Islet RMM expects from the host. The same repository provides the guest kernel, see [third-party/realm-linux](#third-partyrealm-linux).

### third-party/realm-linux

* Repository: https://github.com/islet-project/3rd-linux, branch `cca-guest/v6`
* Upstream: the Linux kernel
* License: **GPL-2.0-only**, `WITH Linux-syscall-note` on the UAPI headers

The realm (guest) Linux kernel.

### third-party/android-kernel

* Repository: https://github.com/islet-project/3rd-android-kernel, branch `main`
* Upstream: the Android common (GKI) kernel
* License: **GPL-2.0-only**, `WITH Linux-syscall-note` on the UAPI headers

Arm CCA enabled Android host and guest (Microdroid) kernels; they are the source of the prebuilt kernel images listed in the [odcc-aosp-modules-virtualization](#odcc-aosp-modules-virtualization)
section.

### third-party/tf-a

* Repository: https://github.com/islet-project/3rd-tf-a, branch `main`
* Upstream: Trusted Firmware-A (https://review.trustedfirmware.org/TF-A/trusted-firmware-a)
* License: **BSD-3-Clause**

The EL3 Secure Monitor and the early boot stages (BL1/BL2/BL31, plus the OP-TEE BL32 wired up by `third-party/optee-build`) that bring the normal world and the RMM up on both platforms, including the RME/RMI platform code our changes add on top of upstream TF-A.

### third-party/tf-a-rss

* Repository: https://github.com/islet-project/3rd-tf-a, branch `hes`
* Upstream: Trusted Firmware-A
* License: **BSD-3-Clause**

The second checkout of the TF-A fork: the branch that enables communication with Islet HES.

### third-party/tf-a-tests

* Repository: https://github.com/islet-project/3rd-tf-a-tests, branch `main`
* Upstream: Trusted Firmware-A Tests (https://review.trustedfirmware.org/TF-A/tf-a-tests.git)
* License: **BSD-3-Clause**

The Trusted Firmware Test Framework payload, booted as the normal world instead of Linux to run the EL1/EL2/EL3 test suites against the Islet RMM.

### third-party/tf-rmm

* Repository: https://github.com/islet-project/3rd-tf-rmm, branch `main`
* Upstream: TF-RMM, the reference RMM of the TrustedFirmware.org Arm CCA stack
* License: **BSD-3-Clause**

The reference RMM, kept as a submodule so a run can be switched between monitors
(`--rmm=islet|trp|tf-rmm`).

### third-party/cca-rmm-acs

* Repository: https://github.com/islet-project/cca-rmm-acs, branch `islet-v1.0-rel0`
* Upstream: Arm CCA RMM ACS, the Arm compliance suite for the RMM interface
  (https://github.com/ARM-software/cca-rmm-acs, Arm doc DEN0137)
* License: **BSD-3-Clause**

The test binary that checks our RMM against the Arm-defined RMI behaviour; our fork tracks the EAC5 release the RMM implements.

### third-party/kvmtool

* Repository: https://github.com/islet-project/3rd-kvmtool, branch `cca/v3`
* Upstream: kvmtool (`lkvm`), https://gitlab.arm.com/linux-arm/kvmtool-cca
* License: **GPL-2.0-only**

The CCA-capable host VMM.

### third-party/kvmtool-rim-measurer

* Repository: https://github.com/islet-project/3rd-kvmtool, branch `kvmtool-rim-measurer/eac5`
* Upstream: kvmtool
* License: **GPL-2.0-only**

The tool that measures the Realm Initial Measurement (RIM) of a realm image. It consumes the same set of option as a kvmtool (lkvm).

### third-party/kvm-unit-tests

* Repository: https://git.gitlab.arm.com/linux-arm/kvm-unit-tests-cca.git, branch
  `cca/rmm-v1.0-eac5` (pinned directly from Arm, not forked into our organization)
* Upstream: the CCA branch of the Linux KVM unit tests published by Arm
* License: **GPL-2.0-only**

Realm-side unit tests: the `.flat` payloads are started inside a realm by `lkvm` to exercise RSI and realm behaviour (including the attestation flow) from guest side.

### third-party/qemu

* Repository: https://github.com/qemu/qemu.git, branch `stable-9.2`
* Upstream: https://github.com/qemu/qemu.git
* License: **GPL-2.0-only**

QEMU with the Arm RME/CCA machine support.

### third-party/mbedtls

* Repository: https://github.com/Mbed-TLS/mbedtls.git, tracked ref `v3.6.4` (the LTS release tag)
* Upstream: https://github.com/Mbed-TLS/mbedtls.git
* License: **Apache-2.0 OR GPL-2.0-or-later**

A portable C library that implements the TLS (Transport Layer Security), DTLS (Datagram TLS), and X.509 certificate protocols, along with a wide range of cryptographic algorithms.

### third-party/dtc

* Repository: https://git.kernel.org/pub/scm/utils/dtc/dtc.git, pinned by SHA (`18aa49a`)
* Upstream: https://git.kernel.org/pub/scm/utils/dtc/dtc.git
* License: **GPL-2.0-or-later**, with `libfdt` **dual-licensed GPL-2.0-or-later OR BSD-2-Clause**

Device tree handling library.

### third-party/certifier

* Repository: https://github.com/ccc-certifier-framework/certifier-framework-for-confidential-computing,
  pinned by SHA, no tracked branch
* Upstream: https://github.com/ccc-certifier-framework/certifier-framework-for-confidential-computing
* License: **Apache-2.0**

The Confidential Computing Certifier Framework is an open-source project designed to simplify and unify programming and operations for multi-vendor confidential computing environments.

### third-party/ciborium

* Repository: https://github.com/islet-project/ciborium, branch `3rd-ciborium`
* Upstream: https://github.com/enarx/ciborium
* License: **Apache-2.0**

CBOR handling for Rust.

### third-party/coset

* Repository: https://github.com/islet-project/coset, branch `3rd-coset`
* Upstream: https://github.com/google/coset (COSE structures for Rust)
* License: **Apache-2.0**

Provides the COSE structures the attestation tokens are encoded in. Our patch links it against the locally checked out [ciborium](#third-partyciborium) fork, so the COSE layer and the CBOR layer of the RMM always come from one and the same pair of forks.

### third-party/cargo-geiger

* Repository: https://github.com/bitboom/cargo-geiger, pinned by SHA (`master`), no tracked branch
* Upstream: https://github.com/bitboom/cargo-geiger
* License: **Apache-2.0 OR MIT**

Cargo Geiger is a plugin for Rust’s package manager (Cargo) that scans your project and its entire dependency tree to measure and report the usage of unsafe Rust code.

### third-party/linux-rsi

It is described in [linux-rsi](#linux-rsi) seciton.

### third-party/optee-build

* Repository: https://github.com/islet-project/assets, branch `3rd-optee-build-cca-v5`
* Upstream: https://github.com/OP-TEE/build
* License: BSD-style license

The OP-TEE build makefiles that `scripts/fvp-cca` (`fvp.mk`) and `scripts/qemu-cca`
(`qemu_v8_cca.mk`) drive: they build the normal-world kernel, the FVP `boot.img` and its GRUB config
and, on QEMU, also the TF-A FIP, tf-rmm, QEMU itself and the QEMU run. The OP-TEE OS and buildroot
parts are unused — `islet` has no BL32, and the initrd and GRUB binary come from `assets`.


## Repositories in the islet-project organization

### linux-rsi

* Repository: https://github.com/islet-project/linux-rsi
* Origin: written by us
* License: GPL-2.0-only

Out-of-tree Linux kernel module that creates `/dev/rsi` and exposes the RSI operations of the Arm CCA Realm Silicon Interface to user space through ioctls (attestation token,
measurement read/extend, sealing key, plus a few Islet extensions). Its `rsi.h` ABI is
mirrored by the hand-written ioctl wrappers of [rust-rsi](#rust-rsi); its users are
[rsictl](#rsictl) and the Islet SDK. It is always built out of tree against a CCA-enabled guest kernel.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| Linux kernel headers and kbuild of a CCA-enabled guest kernel (provides `<asm/rsi.h>`; the module cannot be built against a vanilla kernel) | :heavy_check_mark: | :heavy_check_mark: | GPL-2.0-only | https://github.com/islet-project/3rd-linux (also vendored as `islet/third-party/realm-linux`) |
| Arm GNU Toolchain (used as `CROSS_COMPILE=aarch64-none-linux-gnu-`) | :heavy_check_mark: |  | GPL-3.0-or-later with the GCC Runtime Library Exception (build tool only) | https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads |

### rust-rsi

* Repository: https://github.com/islet-project/rust-rsi
* Origin: written by us
* License: Apache-2.0

Rust library with the (unsafe) bindings for the RSI ioctls implemented by
[linux-rsi](#linux-rsi), safe wrappers on top of them, and the RSI attestation token
parser, verifier and dumper (COSE/CBOR platform and realm tokens, ECDSA P-256/P-384 and
P-521 signature verification). Used by `rsictl`, `ratls`, `realm-verifier`, `rocli`,
`realm-manager` and by the Islet SDK.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| ciborium (CBOR) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/enarx/ciborium |
| coset (COSE) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/google/coset |
| ecdsa, p256, p384, p521 | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/elliptic-curves, https://github.com/RustCrypto/signatures |
| sha2 | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| nix (feature `ioctl`) | :heavy_check_mark: | :heavy_check_mark: | MIT | https://github.com/nix-rust/nix |
| hex | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| cc, bindgen (declared as build dependencies, but there is no `build.rs` — the ioctl numbers and structures of `src/ioctl/kernel.rs` are hand written to match `rsi.h`) | :heavy_check_mark: |  | MIT OR Apache-2.0 (cc), BSD-3-Clause (bindgen) | https://github.com/rust-lang/cc-rs, https://github.com/rust-lang/rust-bindgen |


### rsictl

* Repository: https://github.com/islet-project/rsictl
* Origin: written by us
* License: Apache-2.0

Command line tool for the RSI kernel module: query the RSI ABI version, read and extend
measurements, request an attestation token, fetch sealing key material, and verify token files offline. The `rsictl` binary shipped as a prebuilt in the Virtualization module and in the realm initramfs is built from this repository.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| rust-rsi (our own library) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/rust-rsi |
| clap (derive) | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| rand (challenge generation) | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/rust-random/rand |

### ratls

* Repository: https://github.com/islet-project/ratls
* Origin: written by us
* License: Apache-2.0; the single third-party
  artifact in the repository is the MobileBERT model shipped as the demo payload of
  `tools/ratls-serve`, whose Apache-2.0 provenance and citation request are documented in `NOTICE.txt`.

RA-TLS implementation for Arm CCA. The TLS layer comes from Rustls and remote attestation is done with custom certificate
resolvers and verifiers. The workspace contains the `ratls` library, example client and server, and two tools: `ratls-get` (attested HTTPS client, shipped as the `ratls_get` prebuilt of the Virtualization module) and `ratls-serve` (attested HTTPS server used by the provisioning demos). Token verification is pluggable — `realm-verifier` (our own) and `veraison-verifier` (Veraison backed).

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| rustls, rustls-pemfile | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls, https://github.com/rustls/pemfile |
| rustls-webpki | :heavy_check_mark: | :heavy_check_mark: | ISC | https://github.com/rustls/webpki |
| rcgen | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/rustls/rcgen |
| x509-certificate, bcder, pkcs8, simple_asn1 (X.509 and ASN.1 handling) | :heavy_check_mark: | :heavy_check_mark: | **MPL-2.0** (x509-certificate — file level copyleft), BSD-3-Clause (bcder), Apache-2.0 OR MIT (pkcs8), ISC (simple_asn1) | https://github.com/indygreg/cryptography-rs, https://github.com/NLnetLabs/bcder, https://github.com/RustCrypto/formats, https://github.com/acw/simple_asn1 |
| rsa, rand, sha2, base64, lazy_static, log, env_logger | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/RustCrypto/RSA, https://github.com/rust-random/rand, https://github.com/RustCrypto/hashes, https://github.com/marshallpierce/rust-base64 |
| tokio, tokio-util, tokio-rustls, tokio-vsock, hyper, hyper-util, http, http-body-util, bytes, tower, tower-http, tower-service, axum, axum-extra, url, mime_guess, futures-util, serde, serde_json, clap, async-trait (the `ratls-get` and `ratls-serve` tools) | :heavy_check_mark: | :heavy_check_mark: | MIT / MIT OR Apache-2.0 / Apache-2.0 (`tokio-vsock`) per crate | https://github.com/tokio-rs/tokio, https://github.com/hyperium/hyper, https://github.com/tokio-rs/axum |
| rust-rsi, realm-verifier, veraison-verifier (our own libraries) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project |
| MobileBERT Q&A TensorFlow Lite model (demo payload of `tools/ratls-serve`, see `NOTICE.txt`) |  | :heavy_check_mark: | Apache-2.0 | https://github.com/google-research/google-research/tree/master/mobilebert |


### realm-verifier

* Repository: https://github.com/islet-project/realm-verifier
* Origin: written by us
* License: Apache-2.0

Verifier backend that implements our own realm token verification: it parses the realm
part of a CCA attestation token and compares the embedded measurement claims against the reference values given in its JSON configuration, without involving any external service. It is the `realm` verifier backend of `ratls` / `ratls-serve` and of the Image Registry.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| rust-rsi (our own library — token parsing and verification) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/rust-rsi |
| ratls (our own library — verifier interface) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/ratls |
| serde, serde_json, tinyvec, hex, log | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 (serde, serde_json, hex, log), Zlib OR Apache-2.0 OR MIT (tinyvec) | https://github.com/serde-rs/serde, https://github.com/Lokathor/tinyvec, https://github.com/KokaKiwi/rust-hex |

### veraison-verifier

* Repository: https://github.com/islet-project/veraison-verifier
* Origin: written by us
* License: Apache-2.0

Verifier back end that delegates the appraisal to a Veraison deployment: it opens a
verification session over HTTP(S), sends the CCA attestation token and checks the
assertions of the returned EAR (EAT Attestation Result). It is the `veraison` verifier back end of `ratls` / `ratls-serve` and of the Image Registry; the reference values Veraison compares against are provisioned with [rocli](#rocli).

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| ear (EAR / EAT Attestation Result handling) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/veraison/rust-ear |
| reqwest (default features off, features `blocking`, `json`, `rustls-tls`) | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/seanmonstar/reqwest |
| serde, serde_json, base64, log | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/serde-rs/serde, https://github.com/marshallpierce/rust-base64 |
| rust-rsi, ratls (our own libraries) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project |
| Veraison server (talked to at runtime over the network, not vendored) |  | :heavy_check_mark: | Apache-2.0 | https://github.com/veraison/veraison |

### rocli

* Repository: https://github.com/islet-project/rocli
* Origin: written by us
* License: Apache-2.0

Reference-owner command line tool: from a YAML description plus a live CCA attestation
token it produces the `endorsements.json`, `refvals.json` and `corim.json` inputs that
`cocli` uses to provision a Veraison deployment.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| rust-rsi (our own library — reads and parses the attestation token) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/rust-rsi |
| serde, serde_json, serde_yaml, base64, base64-serde, chrono, uuid, rand, clap | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 per crate, Apache-2.0 for base64-serde | https://github.com/serde-rs/serde, https://github.com/marshallpierce/base64-serde, https://github.com/chronotope/chrono, https://github.com/uuid-rs/uuid |
| cocli (used at runtime to push the generated JSON to Veraison, not vendored) |  | :heavy_check_mark: | Apache-2.0 | https://github.com/veraison/cocli |


### realm-metadata-tool

* Repository: https://github.com/islet-project/realm-metadata-tool
* Origin: written by us
* License: Apache-2.0

Generates the signed metadata block that our sealing-key derivation is bound to: it reads a YAML manifest, serializes the metadata (versioned, semver checked) and signs it with an ECDSA P-384 key. The binary built from this repository is shipped as the
`realm_metadata_tool` prebuilt of the Virtualization module.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| p384 (feature `pem`), sec1 | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/elliptic-curves, https://github.com/RustCrypto/formats |
| bincode, serde, serde-big-array, serde_derive, serde_yaml, array-init | :heavy_check_mark: | :heavy_check_mark: | MIT (bincode), MIT OR Apache-2.0 (the rest) | https://github.com/servo/bincode, https://github.com/serde-rs/serde, https://github.com/est31/serde-big-array, https://github.com/Manishearth/array-init/ |
| semver, thiserror, clap, hex, hexdump | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/dtolnay/semver, https://github.com/dtolnay/thiserror, https://github.com/clap-rs/clap, https://github.com/KokaKiwi/rust-hex, https://github.com/tbu-/hexdump |

### image-registry

* Repository: https://github.com/islet-project/image-registry
* Origin: written by us
* License: Apache-2.0

OCI compatible application registry used by realm application provisioning. `ir-server`
loads and verifies a set of OCI images held on disk (Image Layout specification) and
serves them over the network following the Distribution specification (pull only),
optionally requiring RA-TLS so that only an attested realm can download them; `ir-client` is the client library and example tool the realm side uses to fetch and unpack an application image; `ir-sign` signs images with an ECDSA P-384 key using OCI annotations (the signature formats are SEC1 / SPKI / DER, openssl compatible); `registry/` holds example registry content and a demo application.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| oci-spec | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/containers/oci-spec-rs |
| tokio, tokio-util, tokio-stream, tokio-tar, tokio-rustls, futures, futures-core, futures-util | :heavy_check_mark: | :heavy_check_mark: | MIT (tokio family), MIT OR Apache-2.0 (futures, tokio-rustls), MIT/Apache-2.0 (tokio-tar) | https://github.com/tokio-rs/tokio, https://github.com/rust-lang/futures-rs, https://github.com/vorot93/tokio-tar |
| hyper, hyper-util, http, bytes, axum, axum-extra, tower, tower-http, tower-service, mime, pin-project, pin-project-lite, url, regex, clean-path | :heavy_check_mark: | :heavy_check_mark: | MIT / MIT OR Apache-2.0 / Apache-2.0 OR MIT per crate | https://github.com/hyperium/hyper, https://github.com/tokio-rs/axum, https://github.com/tower-rs/tower-http, https://gitlab.com/foo-jin/clean-path |
| reqwest (default features off, features `blocking`, `json`, `stream`, `rustls-tls`), rustls, rustls-pemfile, async-compression (gzip, zstd), tar, getrandom | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 (reqwest, async-compression, tar, getrandom), Apache-2.0 OR ISC OR MIT (rustls, rustls-pemfile) | https://github.com/seanmonstar/reqwest, https://github.com/rustls/rustls, https://github.com/Nullus157/async-compression |
| sha2, hex, p384, sec1, spki, serde, serde_json, clap, log, env_logger | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 (sha2, hex, serde, serde_json, clap, log, env_logger), Apache-2.0 OR MIT (p384, sec1, spki) | https://github.com/RustCrypto/hashes, https://github.com/RustCrypto/elliptic-curves, https://github.com/RustCrypto/formats |
| ratls, realm-verifier, veraison-verifier, rust-rsi (our own libraries) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project |


### realm-manager

* Repository: https://github.com/islet-project/realm-manager
* Origin: written by us
* License: Apache-2.0

The **application provisioning framework**: the host side daemon **Warden** (`warden/` cargo workspace — `warden_daemon`, `cmd_client`, `client_lib`), the realm side daemon
**app-manager** plus the realm image that runs it (`realm/` — app-manager, initramfs,
kernel and busybox configuration, root CA in `keys/`, `mocked_host` Python script for
running the protocol without the full stack), and the definitions of the two protocols
spoken over vsock (`protocol/warden_client` for the client facing protocol,
`protocol/warden_realm` for Warden ↔ app-manager). Warden launches and manages realms and gives them network and storage resources; app-manager installs, verifies and launches the provisioned applications and derives their disk encryption keys, downloading the application images from the [Image Registry](#image-registry) over RA-TLS.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| tokio, tokio-util, tokio-serde, tokio-vsock, futures, futures-util, async-trait | :heavy_check_mark: | :heavy_check_mark: | MIT (tokio, tokio-util), MIT OR Apache-2.0 (the rest), Apache-2.0 (`tokio-vsock`) | https://github.com/tokio-rs/tokio, https://github.com/rust-vsock/tokio-vsock |
| serde, serde_json, serde_yaml, uuid, thiserror, anyhow, clap, log, env_logger, simple_logger, shlex, hex, hex-literal, const-hex | :heavy_check_mark: | :heavy_check_mark: | MIT / MIT OR Apache-2.0 / Apache-2.0 OR MIT per crate | https://github.com/serde-rs/serde, https://github.com/uuid-rs/uuid, https://github.com/dtolnay/anyhow |
| devicemapper — our fork of `devicemapper-rs` used by app-manager for the encrypted application partitions | :heavy_check_mark: | :heavy_check_mark: | **MPL-2.0** (file level copyleft — relevant when we patch the fork) | https://github.com/islet-project/devicemapper-rs |
| nix, keyutils, hkdf, sha2, oci-spec | :heavy_check_mark: | :heavy_check_mark: | MIT (nix), BSD-3-Clause (keyutils), MIT OR Apache-2.0 (hkdf, sha2), Apache-2.0 (oci-spec) | https://github.com/nix-rust/nix, https://github.com/mathstuf/rust-keyutils, https://github.com/RustCrypto/KDFs/, https://github.com/containers/oci-spec-rs |
| gpt, rtnetlink, tokio-tun, iptables, ipnet (GPT disk image handling and host networking setup of Warden — these crates are not vendored in our workspace, so their licenses have to be taken from the crate pages; the `iptables` crate also drives the system `iptables` binary at runtime, and the daemon expects `dnsmasq`) | :heavy_check_mark: | :heavy_check_mark: | per crate, see https://crates.io ; the `iptables` and `dnsmasq` binaries used at runtime are GPL-2.0-only | https://crates.io/crates/gpt, https://crates.io/crates/rtnetlink, https://www.netfilter.org, https://thekelleys.org.uk/dnsmasq/doc.html |
| rust-rsi, ratls, ir-client, warden_client, warden_realm, utils (our own crates, the last two from this repository) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project |
| Linux kernel (`islet-project/3rd-linux`, branch `cca-guest/v6`), busybox, util-linux, strace, GDB 14.1, QEMU (cloned or downloaded by `realm/Makefile` to build the realm image and the QEMU test environment) | :heavy_check_mark: | :heavy_check_mark: | GPL-2.0-only (kernel, busybox, QEMU), GPL / LGPL mixed per component (strace, util-linux), GPL-3.0-or-later (GDB) — realm image content and development tooling, not linked into our own binaries | https://github.com/islet-project/3rd-linux, https://github.com/mirror/busybox.git, https://github.com/util-linux/util-linux.git, https://github.com/strace/strace.git, https://sourceware.org/pub/gdb/releases/, https://github.com/qemu/qemu.git |
| rsictl, linux-rsi, devicemapper-rs source trees (cloned by `realm/Makefile`) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 (rsictl), GPL-2.0-only (linux-rsi), MPL-2.0 (devicemapper-rs) | https://github.com/islet-project/rsictl, https://github.com/islet-project/linux-rsi, https://github.com/stratis-storage/devicemapper-rs |
| Arm GNU Toolchain 13.2.rel1 (`aarch64-none-linux-gnu`) | :heavy_check_mark: |  | GPL-3.0-or-later with the GCC Runtime Library Exception (build tool only) | https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads |


### rkik-nts

* Repository: https://github.com/islet-project/rkik-nts
* Upstream project: https://github.com/aguacero7/rkik-nts — an RFC 8915 NTS (Network Time Security) client library — forked, not vendored
* License: MIT

NTS time synchronization client used by the protected VMs. Realms and protected VMs have no direct network access, so the fork adds **vsock support**: a `TransportStream`
abstraction that carries the NTS-KE handshake over either TCP or a vsock stream, and a
`VsockDatagramTransport` that frames the NTP packets (hostname, port, payload) so they can be forwarded by the host side vsock datagram proxy. The `nts_vsock_client` prebuilt binary is used by the Virtualization module.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| aes-siv | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/AEADs |
| tokio, tokio-rustls, rustls, rustls-native-certs, rustls-pemfile, webpki-roots | :heavy_check_mark: | :heavy_check_mark: | MIT (tokio), MIT OR Apache-2.0 (tokio-rustls), Apache-2.0 OR ISC OR MIT (rustls, rustls-native-certs, rustls-pemfile), CDLA-Permissive-2.0 (webpki-roots) | https://github.com/tokio-rs/tokio, https://github.com/rustls/rustls, https://github.com/rustls/webpki-roots |
| x509-parser, sha2, chrono, zeroize, thiserror, tracing, serde, serde_json, rand | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 (x509-parser, sha2, chrono, thiserror, serde, serde_json, rand), MIT (tracing), Apache-2.0 OR MIT (zeroize) | https://github.com/rusticata/x509-parser, https://github.com/RustCrypto/utils |
| tokio-vsock, vsock (added by us for the vsock transports) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/rust-vsock/tokio-vsock, https://github.com/rust-vsock/vsock-rs |
| host side vsock stream and datagram proxies (`libvsock_proxy` in the Virtualization module, which the client talks to at runtime) |  | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-aosp-modules-virtualization |


## Repositories in the islet-poject organization related to Confidential Computing on Android project

The project is based on the AOSP 15.0.0_r8 source code. The [islet-project](https://github.com/islet-project) organization contains repositories that have been either created by us or forked from the main AOSP repository and then modified.

### odcc-ccplugin

* Repository: https://github.com/islet-project/odcc-ccplugin
* Origin: implemented by us from scratch (initial commit: "CCPlugIn that allows developers to run
  their Java services in a Realm")
* License: Apache-2.0

CCPlugIn is the plug-in that lets a regular Android bound Java service be executed inside an Arm CCA Realm. It provides the Realm-side Android API mock-ups, the native payload (JVM launcher + Binder bridge), the JNI bindings, the host-side VM lifecycle manager and the logging helpers.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| aidl (CC enabled code generator) | :heavy_check_mark: | | Apache-2.0 | https://github.com/islet-project/odcc-aosp-aidl-tool |
| Own CCPlugIn modules (`libccplugin_islet_realm`, `libccplugin_islet_android`, `libccplugin_android_mockups`, `libccplugin_native`, `libccplugin_jni`, `libccplugin_vm`, `libccplugin_log`) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-ccplugin |
| `ccplugin_aidl` (own `IRealmService` AIDL interface) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-ccplugin |
| libbinder, libbinder_ndk | | :heavy_check_mark: | Apache-2.0 (AOSP module) | https://github.com/islet-project/odcc-aosp-frameworks-native |
| libutils, libcutils, libbase | | :heavy_check_mark: | Apache-2.0 (AOSP module) | https://android.googlesource.com/platform/system/core |
| libnativehelper | | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-aosp-libnativehelper |
| libnativeloader | | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-aosp-art |
| libvm_payload, android.system.virtualization.payload-java (AVF payload API) | | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-aosp-modules-virtualization |
| core-icu4j | | :heavy_check_mark: | Unicode-3.0 (+ Apache-2.0 Android glue) | https://github.com/islet-project/odcc-aosp-external-icu |
| ART runtime (`java.*` classes used by the payload inside the Realm) | | :heavy_check_mark: | Apache-2.0 (ART); the OpenJDK class libraries shipped by AOSP are GPL-2.0-with-classpath-exception | https://github.com/islet-project/odcc-aosp-art |

### odcc-example-aosp

* Repository: https://github.com/islet-project/odcc-example-aosp
* Origin: written by us
* License: Apache-2.0

Reference client/server applications that show how to apply CCPlugIn.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| CCPlugIn (`libccplugin_vm`, `libccplugin_log`, `ccplugin_aidl`, `libccplugin_native`) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-ccplugin |
| aidl (CC enabled code generator) | :heavy_check_mark: | | Apache-2.0 | https://github.com/islet-project/odcc-aosp-aidl-tool |
| androidx.appcompat | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/androidx/androidx |
| androidx.constraintlayout | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/androidx/constraintlayout |
| Material Components for Android | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/material-components/material-components-android |


### odcc-ccservice-calculate-rem

* Repository: https://github.com/islet-project/odcc-ccservice-calculate-rem
* Origin: written by us
* License: Apache-2.0

A development tool that pre-calculates the Realm Extensible Measurement
(REM1) of an Android CC service payload (APK/APEX). This reference value is intended to be used by the attestation verification service.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| `librmm_measure` (our RMM measurement library) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-aosp-modules-virtualization |
| anyhow | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow |
| clap | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| hex | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| ring | :heavy_check_mark: | :heavy_check_mark: | ISC AND (Apache-2.0 OR MIT) | https://github.com/briansmith/ring |
| serde, serde_json | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/serde-rs/serde, https://github.com/serde-rs/json |
| zip | :heavy_check_mark: | :heavy_check_mark: | MIT | https://github.com/zip-rs/zip |
| log (`liblog_rust`), env_logger | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| apkverify, apkmanifest, apexutil (AOSP AVF host libraries) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 (AOSP module) | https://github.com/islet-project/odcc-aosp-modules-virtualization |

### odcc-microdroid-calculate-rem

* Repository: https://github.com/islet-project/odcc-microdroid-calculate-rem
* Origin: written by us
* License: Apache-2.0

Development tool that calculates the measurements (RIM/REM) of the
Microdroid system image used by CC Realms. These values are intended to be used by the attestation verification service.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| `librmm_measure` (our RMM measurement library) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-aosp-modules-virtualization |
| anyhow | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow |
| clap | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| hex | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| ring | :heavy_check_mark: | :heavy_check_mark: | ISC AND (Apache-2.0 OR MIT) | https://github.com/briansmith/ring |
| vbmeta (AOSP AVF host library) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 (AOSP module) | https://android.googlesource.com/platform/packages/modules/Virtualization |

### odcc-tf-lite-bert-qa

* Repository: https://github.com/islet-project/odcc-tf-lite-bert-qa (branch `main` for the apps,
  branch `provisioning` for the Realm provisioning variant)
* Origin: our changes on top of the TensorFlow Lite BERT QA example application
  (https://github.com/tensorflow/examples/tree/master/lite/examples/bert_qa/android), which was split
  into a client app plus a separate bound service and adapted to the AOSP build system
* License: Apache-2.0

Real-life example of an Android CC service: a TensorFlow Lite MobileBERT question-answering service running inside a Realm and reached through a CCPlugIn proxy.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| TensorFlow examples (BERT QA Android sample sources) | | :heavy_check_mark: | Apache-2.0 | https://github.com/tensorflow/examples |
| TensorFlow Lite Task Text 0.3.0 (`tensorflow-lite-task-text-0.3.0.aar`) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/tensorflow/tflite-support |
| TensorFlow Lite Task Base 0.3.0 (`tensorflow-lite-task-base-0.3.0.aar`) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/tensorflow/tflite-support |
| MobileBERT TFLite model (`assets/mobilebert.tflite`) | | :heavy_check_mark: | Apache-2.0 | https://www.kaggle.com/models/tensorflow/mobilebert/tfLite/metadata |
| Gson 2.9.0 (`gson-2.9.0.jar`) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/google/gson |
| CCPlugIn (`libccplugin_vm`, `libccplugin_log`, `ccplugin_aidl`) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-ccplugin |
| aidl (CC enabled code generator) | :heavy_check_mark: | | Apache-2.0 | https://github.com/islet-project/odcc-aosp-aidl-tool |
| androidx.appcompat, androidx.constraintlayout, androidx.navigation | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/androidx/androidx |
| Material Components for Android | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/material-components/material-components-android |

### odcc-aosp-build-release

* Repository: https://github.com/islet-project/odcc-aosp-build-release
* Upstream: AOSP `platform/build/release` — Apache-2.0
* License of this repository: Apache-2.0

The repository contains build configuration files.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| aconfig / release config machinery (`build/make`) | AOSP build-time tooling | Apache-2.0 | https://android.googlesource.com/platform/build/make |
| No third-party dependencies are added by our changes | | | |

### odcc-aosp-system-apex

* Repository: https://github.com/islet-project/odcc-aosp-system-apex
* Upstream: AOSP `platform/system/apex` — Apache-2.0
* License of this repository: Apache-2.0

Our changes: enable host support for the APEX manifest Rust library, which our host-side CC measurement tooling builds against.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| liblog, liblogwrap, libvintf and the other libraries `apexd` already uses in AOSP | AOSP modules | Apache-2.0 | https://android.googlesource.com/platform/system/apex |
| Linux kernel loop and dm-verity/device-mapper interfaces | kernel UAPI | GPL-2.0-only (with Linux-syscall-note) | https://www.kernel.org |
| No new third-party dependencies are added by our changes | | | |

### odcc-aosp-art

* Repository: https://github.com/islet-project/odcc-aosp-art
* Upstream: AOSP `platform/art` — Apache-2.0 (a few upstream modules are BSD or
  GPL-2.0-with-classpath-exception)
* License of this repository: Apache-2.0

Our changes: allow `CCPlugIn` to use `libnativeloader` (visibility), so that a Java CC service can load its DEX payload inside the Realm.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| VIXL (ARM64 assembler used by the ART JIT/compiler) | third-party | BSD-3-Clause | https://git.linaro.org/toolchain/vixl.git |
| zlib | third-party | Zlib | https://github.com/madler/zlib |
| brotli | third-party | MIT | https://github.com/google/brotli |
| protobuf | third-party | BSD-3-Clause | https://github.com/protocolbuffers/protobuf |
| OpenJDK class libraries shipped with the ART APEX | runtime | GPL-2.0-with-classpath-exception | https://android.googlesource.com/platform/libcore |
| libbase, liblog, libnativehelper, … used by `libnativeloader` | AOSP modules | Apache-2.0 | https://android.googlesource.com/platform/art |


### odcc-aosp-bionic

* Repository: https://github.com/islet-project/odcc-aosp-bionic
* Upstream: AOSP `platform/bionic`
* License of this repository: Apache-2.0 with BSD / MIT / ISC / public-domain parts inherited from FreeBSD, OpenBSD and the ARM Optimized Routines

Our changes: add the syscalls that `kvmtool` (`lkvm`) needs to the app `seccomp` allowlist (`libc/SECCOMP_ALLOWLIST_APP.TXT`), plus the matching `linker/linker_phdr.cpp` adjustment.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| Linux kernel UAPI headers | kernel headers | GPL-2.0-only (with Linux-syscall-note) | https://www.kernel.org |
| ARM Optimized Routines (`external/arm-optimized-routines`) | third-party | BSD-3-Clause (MIT-style variants included) | https://github.com/ARM-software/optimized-routines |
| No new dependencies are added by our changes | | | |

### odcc-aosp-device-google-cuttlefish

* Repository: https://github.com/islet-project/odcc-aosp-device-google-cuttlefish
* Upstream: AOSP `device/google/cuttlefish`
* License of this repository: Apache-2.0 (some upstream files BSD-3-Clause / MIT)

Our changes: add the parameters and the launcher changes that are required to boot Cuttlefish on top of a QEMU/Cuttlefish setup that supports Arm CCA — the new `qemu_initrd`, `cpu_feature` and `hes_serial_port` options of `launch_cvd` (the last one exposes the QEMU serial port as a TCP server that the Hardware Enforced Security daemon connects to) and the QEMU-RME activation in the launcher scripts.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| crosvm / QEMU launched by the Cuttlefish host | external VMM | BSD-3-Clause (crosvm) / GPL-2.0-only (QEMU) | https://android.googlesource.com/platform/external/crosvm |
| Arm CCA capable host & guest kernels | external kernel | GPL-2.0-only | https://github.com/islet-project/3rd-android-kernel |
| libbase, libcutils, liblog and the other Cuttlefish host/guest libraries | AOSP modules | Apache-2.0 | https://android.googlesource.com/device/google/cuttlefish |
| No new third-party dependencies are added by our changes | | | |

### odcc-aosp-external-crosvm

* Repository: https://github.com/islet-project/odcc-aosp-external-crosvm
* Upstream: AOSP `platform/external/crosvm` (crosvm, ChromiumOS Virtual Machine Monitor)
* License of this repository: BSD-3-Clause (`LICENSE`), with some Apache-2.0 / MIT files inherited from upstream

Our changes: `export_image_to_raw()` in `disk/src/disk.rs`, which converts a VM disk image in any format crosvm understands (composite disk, qcow2, sparse, raw) into a flat raw image — used by our Realm image build and measurement flow.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| crosvm crates in `external/crosvm` (about 300 third-party Rust crates vendored by AOSP, e.g. `serde`, `tokio`, `libc`, `anyhow`, `thiserror`, `nix`, `byteorder`) | third-party crates | MIT / Apache-2.0 / BSD-3-Clause (per crate) | https://chromium.googlesource.com/chromiumos/platform/crosvm |
| KVM uapi | kernel interface | GPL-2.0-only (with Linux-syscall-note) | https://www.kernel.org |
| No new third-party dependencies are added by our changes | | | |

### odcc-aosp-external-icu

* Repository: https://github.com/islet-project/odcc-aosp-external-icu
* Upstream: AOSP `platform/external/icu` (ICU 75.1) — Unicode License v3 (`LICENSE`), with the
  Android-specific glue under Apache-2.0
* License of this repository: Unicode-3.0 for the ICU sources + Apache-2.0 for the Android glue

Our changes: expose `core-icu4j` to `CCPlugIn` so that the CC service Java library can be built and linked against the same ICU-backed core libraries as the rest of the platform.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| ICU4C / ICU4J 75.1 | third-party | Unicode-3.0 | https://github.com/unicode-org/icu |
| No new dependencies are added by our changes | | | |

### odcc-aosp-external-toybox

* Repository: https://github.com/islet-project/odcc-aosp-external-toybox
* Upstream: AOSP `platform/external/toybox` (toybox, https://github.com/landley/toybox)
* License of this repository: 0BSD

Our changes: enable the command line tools that Microdroid inside a Realm is missing — the DHCP and NTP clients in the Android/Linux tool configs plus a fix of the checksum verification in DHCP.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| toybox | third-party | 0BSD | https://github.com/landley/toybox |
| bionic libc | AOSP | Apache-2.0 | https://github.com/islet-project/odcc-aosp-bionic |
| No new dependencies are added by our changes | | | |


### odcc-aosp-frameworks-native

* Repository: https://github.com/islet-project/odcc-aosp-frameworks-native
* Upstream: AOSP `platform/frameworks/native`
* License of this repository: Apache-2.0 (a few upstream files are MIT / BSD /
  LGPL, e.g. some Vulkan and OpenGL headers)

Our changes: enable the Android runtime Binder support that the CC service proxies need, so that a Binder transaction can be routed through the CCPlugIn bridge into the Realm.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| libbinder, libbinder_ndk | AOSP modules (this repository) | Apache-2.0 | https://github.com/islet-project/odcc-aosp-frameworks-native |
| libutils, libcutils, libbase, liblog | AOSP modules | Apache-2.0 | https://android.googlesource.com/platform/system/core |
| libvm_payload (AVF payload side used by the CC bridge) | AOSP module | Apache-2.0 | https://github.com/islet-project/odcc-aosp-modules-virtualization |
| No new third-party dependencies are added by our changes | | | |

## odcc-aosp-libnativehelper

* Repository: https://github.com/islet-project/odcc-aosp-libnativehelper
* Upstream: AOSP `platform/libnativehelper`
* License of this repository: Apache-2.0

Our changes: Changes in `libnativhelper` enable `CCPlugIn` to statically link the JNI helper symbols into its native payload.

| Name | Kind | License | URL |
| ---- | ---- | ------- | --- |
| JNI/`JNIHelp` headers and sources | upstream sources | Apache-2.0 | https://android.googlesource.com/platform/libnativehelper |
| libbase, liblog | AOSP modules | Apache-2.0 | https://android.googlesource.com/platform/system/core |
| No new dependencies are added by our changes | | | |

### odcc-aosp-modules-virtualization

* Repository: https://github.com/islet-project/odcc-aosp-modules-virtualization
* Upstream: AOSP `platform/packages/modules/Virtualization` (AVF — Android Virtualization Framework)
* License of this repository: Apache-2.0

Components that we implemented in this repository:

* Rust client library for the Arm CCA Realm Services Interface (RSI) kernel driver:
  realm config, RIM/REM measurement read/extend, token ( Realm token) parsing, dumping and verification.
* Arm CCA Support in microdroid: measurement of confidential APK/APEXes (microdroid payload), sealing key derivation and extension of paylaod service with Arm CCA attestation and remote provisioning functionality.
* vsock-to-TCP proxy with a server whitelist and TX byte limit; lets payload VMs without an IP stack do remote attestation/provisioning.
* launching Realms through `lkvm`, running the datagram vsock proxy per VM, integration of the NTS-over-vsock client; support for instance and encrypted disk images.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| nix | | :heavy_check_mark: | MIT | https://github.com/nix-rust/nix |
| ciborium | | :heavy_check_mark: | Apache-2.0 | https://github.com/enarx/ciborium |
| coset | | :heavy_check_mark: | Apache-2.0 | https://github.com/google/coset |
| hex | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| ring | :heavy_check_mark: | :heavy_check_mark: | ISC AND (Apache-2.0 OR MIT) | https://github.com/briansmith/ring |
| vsock | | :heavy_check_mark: | Apache-2.0 | https://github.com/rust-vsock/vsock-rs |
| log (`liblog_rust`) | | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| env_logger | | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/rust-cli/env_logger |
| thiserror | | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| clap | | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| serde, serde_json | | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/serde-rs/serde, https://github.com/serde-rs/json |
| openssl (`libopenssl`, used by `arm_cca.rs` for RIM/REM hashing) | | :heavy_check_mark: | Apache-2.0 AND OpenSSL (AOSP module) | https://android.googlesource.com/platform/external/openssl |
| AVF libraries used by the code we modified (`libvm_control`, `libvmconfig`, `libmicrodroid_metadata`, `libbinder_rs`, `libvsock`, `libapkverify`, `libvbmeta_rust`, …) | | :heavy_check_mark: | Apache-2.0 (AOSP module) | https://github.com/islet-project/odcc-aosp-modules-virtualization |
| kvmtool (`lkvm`) prebuilt binary | | :heavy_check_mark: | GPL-2.0-only | https://git.kernel.org/pub/scm/linux/kernel/git/will/kvmtool.git |
| Arm CCA host & guest kernels (used to produce the prebuilt kernel images) | :heavy_check_mark: | :heavy_check_mark: | GPL-2.0-only | https://github.com/islet-project/3rd-android-kernel |
| `ratls-get`, `nts_vsock_client`, `rsictl`, `realm-metadata-tool`, `lkvm-rim-measurer-static` prebuilt helper binaries | :heavy_check_mark: | :heavy_check_mark: | built by us — Apache-2.0 except where the source repository says otherwise | [ratls](#ratls) (`ratls-get`), [rkik-nts](#rkik-nts) (`nts_vsock_client`), [rsictl](#rsictl), [realm-metadata-tool](#realm-metadata-tool) |



### odcc-aosp-system-core

* Repository: https://github.com/islet-project/odcc-aosp-system-core
* Upstream: AOSP `platform/system/core` —
* License of this repository: Apache-2.0 (a few upstream files are BSD / MIT)

Our changes: `init` support for Arm CCA — extending REM0 with the values that the CC service configuration contributes and deriving the sealing key used to protect confidential data.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| init libraries and the libraries `init` links against (`libbase`, `libcutils`, `liblog`, `libutils`, `libfs_mgr`, `liblp`, `libavb`, …) | | :heavy_check_mark: | Apache-2.0 (AOSP modules) | https://android.googlesource.com/platform/system/core |
| libselinux | | :heavy_check_mark: | public domain | https://github.com/SELinuxProject/selinux |
| RSI (`/dev/rsi`) and sealing-key kernel interfaces of the CCA-enabled kernel | :heavy_check_mark: | :heavy_check_mark: | GPL-2.0-only (kernel) | https://github.com/islet-project/3rd-android-kernel |

### odcc-aosp-system-sepolicy

* Repository: https://github.com/islet-project/odcc-aosp-system-sepolicy
* Upstream: AOSP `platform/system/sepolicy`
* License of this repository: public domain for the policy, as upstream

Our changes: SELinux policy for the whole Realm lifecycle — the new `kvmtool` domain, RSI driver access, guest networking (DHCP/NTP), `ratls-get`/`nts_vsock_client` domains, the datagram vsock proxy and DEX payload execution under the Dalvik VM.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| libsepol, libselinux, secilc, checkpolicy (SELinux userspace used to build/enforce the policy) | :heavy_check_mark: | :heavy_check_mark: | LGPL-2.1-only (libsepol), public domain (libselinux), BSD-3-Clause (secilc), GPL-2.0-only (checkpolicy) | https://github.com/SELinuxProject/selinux |
| Policy of the AVF / Microdroid / init domains that our rules refer to | | | Apache-2.0 / public domain | https://github.com/islet-project/odcc-aosp-modules-virtualization |

### odcc-aosp-aidl-tool

* Repository: https://github.com/islet-project/odcc-aosp-aidl-tool
* Upstream: AOSP `platform/system/tools/aidl`
* License of this repository: Apache-2.0

Our changes: `CCProxy` / `CCStub` code generation for Arm CCA Realms — generating the Java CC proxy, the target (Realm) service package and the manifest fragment that `CCPlugIn` uses, driven by the `@GenerateCCService` annotation.

| Name | Build time | Runtime | License | URL |
| ---- | :--------: | :-----: | ------- | --- |
| libaidl-common (the `aidl` library in this repository) | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 | https://github.com/islet-project/odcc-aosp-aidl-tool |
| libbase, liblog | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 (AOSP modules) | https://android.googlesource.com/platform/system/core |
| flex / bison generated AIDL grammar (unchanged from upstream, build time only) | :heavy_check_mark: | | as in upstream AOSP | https://android.googlesource.com/platform/system/tools/aidl |

