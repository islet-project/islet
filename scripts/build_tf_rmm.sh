#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

cd $ROOT/third-party/tf-rmm
# git submodule update --init --recursive
export CROSS_COMPILE=$1
cmake -DRMM_CONFIG=fvp_defcfg -S . -B build
cmake --build build

cp build/rmm.img $ROOT/out/tf-rmm.img
