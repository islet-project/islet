#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
CROSS_COMPILE=${ROOT}/assets/toolchain/gcc-aarch64-none-elf/bin/aarch64-none-elf-
TF_A_TESTS=${ROOT}/tf-a-tests/
TRUSTED_FIRMWARE_A=${ROOT}/trusted-firmware-a/
VM_IMAGE=${ROOT}/vm-image/
FIPTOOL=${TRUSTED_FIRMWARE_A}/tools/fiptool/fiptool
FASTMODEL=${ROOT}/assets/fastmodel/
RMM=${ROOT}/rmm/bin/
