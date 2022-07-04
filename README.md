# islet
ISLET is a project to enable on-device confidential computing for end users by leveraging ARMv9 CCA that is the newly emerging confidential computing hardware on ARM devices. Using the hardware support, ISLET enables a Trusted Execution Environment (TEE) on user’s devices within which users can securely process, store, communicate and manage their private data. The protection provided by ISLET applies not only to data-at-rest but also to data-in-use even in the presence of malicious privileged software on devices.  We develop components enabling Realm Virtual Machines (VMs), which are secure VM-level TEE provided by ARMv9 CCA. To manage Realm VMs, Realm Management Monitor (RMM) is needed to be running at EL2 in the Realm world. Although the Monitor firmware is available as an open source, there is no functional Realm Management Monitor (RMM) code available yet. ISLET provides the implementation of RMM that is written in Rust. 

<!---
## How to get prerequisites for build

```bash
./scripts/prepare_qemu.sh
./scripts/prepare_fastmodel.sh
./scripts/prepare_toolchains.sh
```

```bash
cd assets/prebuilt/qemu/
unzip qemu-system-aarch64.zip
cd -
```

```bash
mkdir -p assets/fastmodel
cd assets/fastmodel
wget https://developer.arm.com/-/media/Files/downloads/ecosystem-models/FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz
tar -xzf FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz
cd -
```

```bash
mkdir -p assets/toolchains
cd assets/toolchains
wget https://developer.arm.com/-/media/Files/downloads/gnu-a/10.2-2020.11/binrel/gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz
wget https://developer.arm.com/-/media/Files/downloads/gnu-a/10.2-2020.11/binrel/gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz
tar -xf gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz
tar -xf gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz
ln -sf gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu aarch64
ln -sf gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf aarch32
cd -
scripts/prepare_toolchains.sh
```
--->


## How to prepare build
```bash
./scripts/init.sh
```

## How to run
### Start FVP
```bash
./scripts/fvp-cca --normal-world={linux|tf-a-tests} --realm-vm={tftf|linux}
./scripts/fvp-cca --nw={linux|tf-a-tests| -vm={tftf|linux}
```
### Login with root in the normal world linux
```bash
Welcome to Buildroot, type root or test to login
buildroot login: root
```

### Run a tftf realm
```bash
# cd /qemu/guest/
# ../qemu-system-aarch64 \
    -kernel tftf-realm.elf \
    --enable-kvm \
    -cpu host \
    -smp 1 \
    -m 256M \
    -M virt,gic-version=3 \
    -nographic
```

### Run a linux realm
```bash
../qemu-system-aarch64 \
        -kernel Image_realmvm \
        -initrd initramfs-busybox-aarch64.cpio.gz \
        -append "earlycon=pl011,mmio,0x1c0a0000 console=ttyAMA0" \
        --enable-kvm \
        -cpu host \
        -smp 1 \
        -M virt,gic-version=3 \
        -m 256M \
        -nographic
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
