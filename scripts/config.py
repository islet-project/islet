#!/usr/bin/env python3

import os

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "out")
CONFIG = os.path.join(ROOT, "scripts/.config")

PREBUILT = os.path.join(ROOT, "assets/prebuilt")
PREBUILT_EDK2 = os.path.join(PREBUILT, "FVP_AARCH64_EFI.fd")
PREBUILT_GRUB = os.path.join(PREBUILT, "bootaa64.efi")
PREBUILT_QEMU = os.path.join(PREBUILT, "qemu")

REALM = os.path.join(ROOT, "realm")
RMM = os.path.join(ROOT, "rmm/board/fvp")
SDK = os.path.join(ROOT, "sdk/")

# third-party
THIRD_PARTY = os.path.join(ROOT, "third-party")

BUILD_SCRIPT = os.path.join(THIRD_PARTY, "optee-build")
GUEST_LINUX = os.path.join(THIRD_PARTY, "realm-linux")
NW_LINUX = os.path.join(THIRD_PARTY, "nw-linux")
TF_A = os.path.join(THIRD_PARTY, "tf-a")
TF_A_TESTS = os.path.join(THIRD_PARTY, "tf-a-tests")

# directory shared between the pc desktop and the host OS on fvp using 9p filesystem
PC_SHARE_DIR = os.path.join(OUT, "pc_share_dir")

#CROSS_COMPILE = os.path.join(ROOT, "assets/toolchains/aarch64/bin/aarch64-none-linux-gnu-")
CROSS_COMPILE = os.path.join(ROOT, "assets/gcc/arm-gnu-toolchain-11.3.rel1-x86_64-aarch64-none-elf/bin/aarch64-none-elf-")
LINUX_CROSS_COMPILE = os.path.join(ROOT, "assets/toolchains/aarch64/bin/aarch64-linux-gnu-")
FASTMODEL = os.path.join(ROOT, "assets/fastmodel/Base_RevC_AEMvA_pkg/models/Linux64_GCC-9.3")
FIPTOOL = os.path.join(TF_A, "tools/fiptool")
