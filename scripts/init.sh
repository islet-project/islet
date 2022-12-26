#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

sudo apt install -y -qq --no-install-recommends \
	binutils python3-pip \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	openjdk-11-jre \
	flex bison

pip3 install toml

# Sync thirt-party projects as worktree
$ROOT/scripts/sync-worktree.py
if [ $? -ne 0 ]; then
	echo "Failed to sync worktree."
	exit 1
fi

# Sync islet-assets
$HERE/deps/assets.sh
if [ $? -ne 0 ]; then
	echo "Failed to sync assets."
	exit 1
fi

# Install rust
$HERE/deps/rust.sh

#pip3 install pre-commit
#pre-commit install

echo "preparing prerequisites for build"
cd ${ROOT}
./scripts/prepare_qemu.sh
./scripts/prepare_fastmodel.sh
./scripts/prepare_toolchains.sh
