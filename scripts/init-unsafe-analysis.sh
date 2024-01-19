#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts
TOOL=$ROOT/third-party/cargo-geiger

$HERE/deps/rust.sh

git submodule update --init $TOOL
cargo +stable install cargo-geiger --force --locked \
	--path $TOOL/cargo-geiger \
	--target x86_64-unknown-linux-gnu
