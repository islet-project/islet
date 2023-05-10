#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

# Install packages
$HERE/deps/pkgs.sh

# Sync thirt-party projects as worktree
$ROOT/scripts/sync-worktree.py

# Sync assets
$HERE/deps/assets.sh

# Sync submodule of third-party
$HERE/deps/submodule.sh

# Install rust
$HERE/deps/rust.sh

echo "preparing prerequisites for build"
cd ${ROOT}
./scripts/prepare_qemu.sh
