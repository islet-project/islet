#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

# Install packages
$HERE/deps/pkgs.sh

# Sync submodules (only assets and tf-a)
git submodule update --init --depth 1 $ROOT/assets
git submodule update --init --depth 1 $ROOT/third-party/tf-a

# Install rust
$HERE/deps/rust.sh

# Install FVP simulator
$HERE/deps/simulator.sh
