#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts
TOOL=$ROOT/third-party/utrace

$HERE/deps/rust.sh

# MIRI Setup
rustup component add --toolchain nightly-2024-04-21-x86_64-unknown-linux-gnu miri

# Utrace Setup
rm -rf $TOOL
cd $ROOT/third-party
git clone https://github.com/islet-project/utrace.git
cd $TOOL && make init
