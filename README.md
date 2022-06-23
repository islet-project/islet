# islet
ISLET is a project to enable on-device confidential computing for end users by leveraging ARMv9 CCA that is the newly emerging confidential computing hardware on ARM devices. Using the hardware support, ISLET enables a Trusted Execution Environment (TEE) on user’s devices within which users can securely process, store, communicate and manage their private data. The protection provided by ISLET applies not only to data-at-rest but also to data-in-use even in the presence of malicious privileged software on devices.  We develop components enabling Realm Virtual Machines (VMs), which are secure VM-level TEE provided by ARMv9 CCA. To manage Realm VMs, Realm Management Monitor (RMM) is needed to be running at EL2 in the Realm world. Although the Monitor firmware is available as an open source, there is no functional Realm Management Monitor (RMM) code available yet. ISLET provides the implementation of RMM that is written in Rust. 

## How to prepare build
```bash
./scripts/init.sh
```

## How to run
```bash
./scripts/fvp-cca --normal-world=linux
./scripts/fvp-cca --normal-world=tf-a-tests
```

## How to do unit-tests
```bash
./scripts/test.sh --unit-test
```

## How to measure line coverage of unit-tests
```bash
./scripts/test.sh --coverage
```

## How to connect T32
```bash
./scripts/fvp-cca --normal-world=tf-a-tests --debug
```

Then, execute the t32 application (e.g., ./t32marm-qt)
and run the script ./debug/t32.cmm via "File -> Run Script".

## Coding style
For bash scripts,
```bash
assets/formatter/shfmt -w -ci -bn -fn <TARGET>
```

For rust,
```bash
cargo fmt
```

.editorconfig is also ready as well.

This file helps use proper indentation when you use editor (e.g., vim, vscode).

You can set the editor configuration like the below if you use vim.

[How to use .editorconfig for vim](https://github.com/editorconfig/editorconfig-vim)

