#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

sudo apt update
sudo apt install -y -qq --no-install-recommends \
	binutils python3-pip \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	flex bison

pip3 install toml

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
