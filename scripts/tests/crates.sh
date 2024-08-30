#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

mkdir -p $ROOT/out

cd $ROOT/rmm
cross test $1 \
	--target=aarch64-unknown-linux-gnu \
	--lib -- --test-threads=1 --nocapture

if [ $? -ne 0 ]; then
	exit 1
fi
