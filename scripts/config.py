#!/usr/bin/env python3

import os

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "out")

SCRIPT = os.path.join(ROOT, "scripts")
CONFIG = os.path.join(SCRIPT, ".config")
LAUNCH_REALM = os.path.join(SCRIPT, "fvp/launch-realm.sh")
TEST_REALM = os.path.join(SCRIPT, "fvp/test-realm.sh")
CONFIGURE_NET = os.path.join(SCRIPT, "fvp/configure-net.sh")
SET_REALM_IP = os.path.join(SCRIPT, "fvp/set-realm-ip.sh")

PREBUILT = os.path.join(ROOT, "assets/prebuilt")
PREBUILT_EDK2 = os.path.join(PREBUILT, "FVP_AARCH64_EFI.fd")
PREBUILT_GRUB = os.path.join(PREBUILT, "bootaa64.efi")
PREBUILT_QEMU = os.path.join(PREBUILT, "qemu")
PREBUILT_AOSP_DTB = os.path.join(PREBUILT, "aosp/fvp-base-aosp.dtb")
PREBUILT_AOSP_INITRD = os.path.join(PREBUILT, "aosp/initrd-aosp.img")
PREBUILT_AOSP_ADB = os.path.join(PREBUILT, "aosp/bind_to_localhost.so")
PREBUILT_EXAMPLES = os.path.join(PREBUILT, "examples")

REALM_ROOTFS = os.path.join(ROOT, "assets/rootfs")

REALM = os.path.join(ROOT, "realm")
RMM = os.path.join(ROOT, "plat/fvp")
SDK = os.path.join(ROOT, "sdk/")
RSI_KO = os.path.join(ROOT, "realm/linux-rsi")
EXAMPLES = os.path.join(ROOT, "examples")

# third-party
THIRD_PARTY = os.path.join(ROOT, "third-party")

ACS = os.path.join(THIRD_PARTY, "cca-rmm-acs")
ACS_BUILD = os.path.join(ACS, "build")
ACS_HOST = os.path.join(ACS, "build/output/acs_host.bin")
ACS_RUN = os.path.join(ACS, "tools/scripts/run.sh")
BUILD_SCRIPT = os.path.join(THIRD_PARTY, "optee-build")
REALM_LINUX = os.path.join(THIRD_PARTY, "realm-linux")
NW_LINUX = os.path.join(THIRD_PARTY, "nw-linux")
NW_AOSP_BUILD_SCRIPT = os.path.join(THIRD_PARTY, "gki-build")
NW_AOSP_OUT = "out/aosp_nw"
MBEDTLS = os.path.join(THIRD_PARTY, "mbedtls")
TF_A = os.path.join(THIRD_PARTY, "tf-a")
TF_A_TESTS = os.path.join(THIRD_PARTY, "tf-a-tests")
TFTF_BIN = os.path.join(TF_A_TESTS, "build/fvp/debug/tftf.bin")
TF_RMM = os.path.join(THIRD_PARTY, "tf-rmm")
KVMTOOL = os.path.join(THIRD_PARTY, "kvmtool")
KVM_UNIT_TESTS = os.path.join(THIRD_PARTY, "kvm-unit-tests")

RSI_TEST = os.path.join(ROOT, "realm/rsi-test")
RSI_TEST_BIN = os.path.join(OUT, "rsi-test.bin")

# directory shared between the pc desktop and the host OS on fvp using 9p filesystem
SHARED_PATH = os.path.join(OUT, "shared")
SHARED_EXAMPLES_PATH = os.path.join(OUT, "shared/examples/")
AOSP_SHARED_PATH = os.path.join(OUT, "shared-aosp")

DTC = os.path.join(ROOT, "assets/dtc")
CROSS_COMPILE = os.path.join(ROOT, "assets/toolchain/aarch64-none-elf/bin/aarch64-none-elf-")
LINUX_CROSS_COMPILE = os.path.join(ROOT, "assets/toolchain/aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-")
KVMTOOL_CROSS_COMPILE = os.path.join(ROOT, "assets/toolchain/aarch64-none-linux-gnu-10-2/bin/aarch64-none-linux-gnu-")
FVP_DIR = os.path.join(THIRD_PARTY, "fvp/Base_RevC_AEMvA_pkg/models/Linux64_GCC-9.3")
FVP_BIN = os.path.join(FVP_DIR, "./FVP_Base_RevC-2xAEMvA")
TRACE_PATH = os.path.join(ROOT, "assets/fastmodel/Base_RevC_AEMvA_pkg/plugins/Linux64_GCC-9.3/TarmacTrace.so")
