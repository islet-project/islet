#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
CROSS_COMPILE=${ROOT}/assets/toolchains/aarch64/bin/aarch64-none-linux-gnu-
TF_A_TESTS=${ROOT}/tf-a-tests/
TRUSTED_FIRMWARE_A=${ROOT}/trusted-firmware-a/
VM_IMAGE=${ROOT}/vm-image/
FIPTOOL=${TRUSTED_FIRMWARE_A}/tools/fiptool/fiptool
FASTMODEL=${ROOT}/assets/fastmodel/
RMM=${ROOT}/rmm/board/fvp
