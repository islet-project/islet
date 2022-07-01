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

if [[ ${1} != "--no-submodule" ]]; then
	cd ${ROOT} \
		&& git submodule update --init --recursive
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

cd ${ROOT} \
    && ./scripts/prepare_qemu.sh \
    && ./scripts/prepare_fastmodel.sh \
    && ./scripts/prepare_toolchains.sh

cd ${ROOT}/trusted-firmware-a \
   && git am -3 ../assets/trusted-firmware-a/0001-add-0x8_8000_0000-dram-for-nw.patch

