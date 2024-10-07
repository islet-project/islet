#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

mkdir -p $ROOT/out

cd $ROOT/rmm

RUSTFLAGS="-Awarnings" MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo miri test $1 \
	--target aarch64-unknown-linux-gnu -- --nocapture

if [ $? -ne 0 ]; then
	exit 1
fi
