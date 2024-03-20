#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

# Install packages
$HERE/deps/pkgs-for-ci.sh

# Sync submodules (only assets, tf-a, tf-a-tests, tf-rmm, mbedtls, ciborium, and coset)
git submodule update --init --depth 1 $ROOT/assets
git submodule update --init --depth 1 $ROOT/third-party/tf-a-rss
git submodule update --init --depth 1 $ROOT/third-party/tf-a-tests
git submodule update --init --recursive --depth 1 $ROOT/third-party/tf-rmm
git submodule update --init --depth 1 $ROOT/third-party/mbedtls
git submodule update --init --depth 1 $ROOT/third-party/ciborium
git submodule update --init --depth 1 $ROOT/third-party/coset

# Install rust (min)
$HERE/deps/rust_min.sh

# Install FVP simulator
$HERE/deps/simulator.sh
