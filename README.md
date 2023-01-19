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

## REALM
ISLET provides sample realms running on fvp.
You may run according to [Getting started](#getting-started)

```
realm/
├── linux
│   └── Makefile
└── wasm
    ├── Makefile
    └── README.md
```

## Getting started 
### Installing dependencies
```bash
./scripts/init.sh
```

### Running the linux realm
```bash
// Start FVP
$ ./scripts/fvp-cca --normal-world=linux --realm=linux

// Login with root in the normal world linux
Welcome to Buildroot, type root or test to login
buildroot login: root

// Run a linux realm
# cd /qemu/guest/
# ../qemu-system-aarch64 \
        -kernel linux.realm \
        -initrd initramfs-busybox-aarch64.cpio.gz \
        -append "earlycon=pl011,mmio,0x1c0a0000 console=ttyAMA0" \
        --enable-kvm \
        -cpu host \
        -smp 1 \
        -M virt,gic-version=3 \
        -m 256M \
        -nographic
```

### Running the wasm realm
```bash
// Start FVP
$ ./scripts/fvp-cca --normal-world=linux --realm=wasm

// Login with root in the normal world linux
Welcome to Buildroot, type root or test to login
buildroot login: root

// Run a wasm realm
# cd /qemu/guest/
# ../qemu-system-aarch64 \
        -kernel linux.realm \
        -initrd wasm-realm-initrd.cpio.gz \
        -append "earlycon=pl011,mmio,0x1c0a0000 console=ttyAMA0" \
        --enable-kvm \
        -cpu host \
        -smp 1 \
        -M virt,gic-version=3 \
        -m 256M \
        -nographic

// Run a wasm on realm
Welcome to wasm realm!
# wasmer ./app/hello.wasm
hello, world!
```

### Testing islet-rmm with tf-a-tests
```
$ ./scripts/fvp-cca --normal-world=tf-a-tests
```
