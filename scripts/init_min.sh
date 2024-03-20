#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

# Install packages
$HERE/deps/pkgs-for-ci.sh

# Sync submodules (only assets, tf-a, ciborium, and coset)
git submodule update --init --depth 1 $ROOT/assets
git submodule update --init --depth 1 $ROOT/third-party/tf-a
git submodule update --init --depth 1 $ROOT/third-party/ciborium
git submodule update --init --depth 1 $ROOT/third-party/coset

# Install rust
$HERE/deps/rust.sh

# Install FVP simulator
$HERE/deps/simulator.sh
