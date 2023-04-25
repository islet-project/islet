#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)

cd $ROOT/third-party/tf-rmm
export CROSS_COMPILE=$1
cmake -DRMM_CONFIG=fvp_defcfg -S . -B build
cmake --build build

cp build/Release/rmm.img $ROOT/out/tf-rmm.img
