#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
RMM=$ROOT/rmm
OUT=$ROOT/out

cd $RMM
cross clean
cargo clean
RUSTFLAGS="-C instrument-coverage" cross test --target=aarch64-unknown-linux-gnu --lib

rm -rf ../code-coverage
grcov . -s . --binary-path $OUT/aarch64-unknown-linux-gnu/debug/ -t html --ignore tests/ -o ../code-coverage
