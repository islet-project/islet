#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

mkdir -p $ROOT/out

cd $ROOT/rmm/monitor
cargo test --lib -- --test-threads=1

if [ $? -ne 0 ]; then
	exit 1
fi
