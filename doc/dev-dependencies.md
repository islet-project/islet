## Islet RMM dependencies
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


## Islet SDK dependencies
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
| sha2     | :heavy_check_mark: | :heavy_check_mark: | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes                           |
| zeroize  | :heavy_check_mark: | :heavy_check_mark: | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils                            |


## Islet HES dependencies
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

