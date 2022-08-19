#!/usr/bin/env python3

import os

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "out")
CONFIG = os.path.join(ROOT, "scripts/.config")
PREBUILT = os.path.join(ROOT, "assets/prebuilt")
GUEST_DEFCONFIG_DIR = os.path.join(ROOT, "assets/guest-linux")

RMM = os.path.join(ROOT, "rmm/board/fvp")
SDK = os.path.join(ROOT, "sdk/")
TF_A_TESTS = os.path.join(ROOT, "tf-a-tests")
TRUSTED_FIRMWARE_A = os.path.join(ROOT, "trusted-firmware-a")
BUILD_SCRIPT = os.path.join(ROOT, "build")

# directory shared between the pc desktop and the host OS on fvp using 9p filesystem
PC_SHARE_DIR = os.path.join(OUT, "pc_share_dir")

CROSS_COMPILE = os.path.join(ROOT, "assets/toolchains/aarch64/bin/aarch64-none-linux-gnu-")
LINUX_CROSS_COMPILE = os.path.join(ROOT, "assets/toolchains/aarch64/bin/aarch64-linux-gnu-")
FASTMODEL = os.path.join(ROOT, "assets/fastmodel/Base_RevC_AEMvA_pkg/models/Linux64_GCC-9.3")
FIPTOOL = os.path.join(TRUSTED_FIRMWARE_A, "tools/fiptool")
