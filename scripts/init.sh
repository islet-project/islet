#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

sudo apt install -y -qq --no-install-recommends \
	git-lfs binutils python3-pip \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	openjdk-11-jre \
	flex bison

# Sync thirt-party projects as worktree
$ROOT/scripts/sync-worktree.py
if [ $? -ne 0 ]; then
	echo "Failed to sync worktree."
	exit 1
fi

# Sync islet-assets
cd ${ROOT} && git submodule update --init --recursive
if [ $? -ne 0 ]; then
	echo "Failed to sync assets."
	exit 1
fi

rustup default nightly && rustup update
cargo install \
	mdbook mdbook-plantuml \
	cargo2junit \
	cargo-tarpaulin

cd ${ROOT} \
	&& rustup component add rustfmt

(
	cd ${ROOT}/rmm/board/fvp
	rustup toolchain install $(cat ${ROOT}/rmm/rust-toolchain)
	rustup target add aarch64-unknown-none-softfloat
	rustup component add rust-src
)

#pip3 install pre-commit
#pre-commit install

echo "preparing prerequisites for build"
cd ${ROOT}
./scripts/prepare_qemu.sh
./scripts/prepare_fastmodel.sh
./scripts/prepare_toolchains.sh
