#!/usr/bin/env python3

import os

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "out")

SCRIPT = os.path.join(ROOT, "scripts")
CONFIG = os.path.join(SCRIPT, ".config")
LAUNCH_REALM = os.path.join(SCRIPT, "launch_realm_on_fvp.sh")

PREBUILT = os.path.join(ROOT, "assets/prebuilt")
PREBUILT_EDK2 = os.path.join(PREBUILT, "FVP_AARCH64_EFI.fd")
PREBUILT_GRUB = os.path.join(PREBUILT, "bootaa64.efi")
PREBUILT_QEMU = os.path.join(PREBUILT, "qemu")
PREBUILT_AOSP_DTB = os.path.join(PREBUILT, "aosp/fvp-base-aosp.dtb")
PREBUILT_AOSP_INITRD = os.path.join(PREBUILT, "aosp/initrd-aosp.img")
PREBUILT_AOSP_ADB = os.path.join(PREBUILT, "aosp/bind_to_localhost.so")

REALM_ROOTFS = os.path.join(ROOT, "assets/rootfs")

REALM = os.path.join(ROOT, "realm")
RMM = os.path.join(ROOT, "rmm/board/fvp")
SDK = os.path.join(ROOT, "sdk/")

# third-party
THIRD_PARTY = os.path.join(ROOT, "third-party")

BUILD_SCRIPT = os.path.join(THIRD_PARTY, "optee-build")
REALM_LINUX = os.path.join(THIRD_PARTY, "realm-linux")
NW_LINUX = os.path.join(THIRD_PARTY, "nw-linux")
NW_AOSP_BUILD_SCRIPT = os.path.join(THIRD_PARTY, "gki-build")
NW_AOSP_OUT = "out/aosp_nw"
TF_A = os.path.join(THIRD_PARTY, "tf-a")
TF_A_TESTS = os.path.join(THIRD_PARTY, "tf-a-tests")
TFTF_BIN = os.path.join(TF_A_TESTS, "build/fvp/debug/tftf.bin")
TF_RMM = os.path.join(THIRD_PARTY, "tf-rmm")
KVMTOOL = os.path.join(THIRD_PARTY, "kvmtool")

# directory shared between the pc desktop and the host OS on fvp using 9p filesystem
PC_SHARE_DIR = os.path.join(OUT, "pc_share_dir")
GUEST_SHARED = os.path.join(PC_SHARE_DIR, "guest")
NW_AOSP_DATA = os.path.join(OUT, "aosp_data")

DTC = os.path.join(ROOT, "assets/dtc")
CROSS_COMPILE = os.path.join(ROOT, "assets/toolchain/aarch64-none-elf/bin/aarch64-none-elf-")
LINUX_CROSS_COMPILE = os.path.join(ROOT, "assets/toolchain/aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-")
KVMTOOL_CROSS_COMPILE = os.path.join(ROOT, "assets/toolchain/aarch64-none-linux-gnu-10-2/bin/aarch64-none-linux-gnu-")
FASTMODEL = os.path.join(ROOT, "assets/fastmodel/Base_RevC_AEMvA_pkg/models/Linux64_GCC-9.3")
