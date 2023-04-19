#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
KVM_UNIT_TESTS=$ROOT/third-party/kvm-unit-tests
TOOLCHAIN="$ROOT/assets/toolchain"
AARCH64="$TOOLCHAIN/aarch64-none-linux-gnu-10-2/bin/aarch64-none-linux-gnu-"

cd $KVM_UNIT_TESTS
#./configure --arch=arm64 --cross-prefix=$AARCH64 --target=kvmtool

make
cp -R arm/* $ROOT/out/shared/kvm-unit-tests
