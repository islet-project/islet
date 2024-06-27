#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

mkdir -p $ROOT/out

cd $ROOT/rmm

MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo miri test --target aarch64-unknown-linux-gnu

if [ $? -ne 0 ]; then
	exit 1
fi
