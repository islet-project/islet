#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CROSS_COMPILE=$ROOT/assets/toolchain/aarch64-none-elf/bin/aarch64-none-elf-

cd $ROOT/third-party/cca-rmm-acs
mkdir -p build && cd build

cmake ../ -G"Unix Makefiles" -DCROSS_COMPILE=$CROSS_COMPILE -DTARGET=tgt_tfa_fvp -DTEST_COMBINE=ON -DSREC_CAT=/bin/srec_cat
make
