#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
TOOL=$ROOT/third-party/cargo-geiger
OUT=$ROOT/out

mkdir -p $OUT

cd $ROOT/plat/fvp
cargo geiger --output-format Ratio | tee $OUT/unsafe-analysis.log
