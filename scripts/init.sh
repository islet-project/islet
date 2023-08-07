#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

# Install packages
$HERE/deps/pkgs.sh

# Sync submodules
$HERE/deps/submodule.sh

# Install rust
$HERE/deps/rust.sh

# Install FVP simulator
$HERE/deps/simulator.sh
