#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

sudo apt install -y -qq --no-install-recommends \
	git-lfs binutils \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	openjdk-11-jre \
	flex bison

if [[ ${1} != "--no-submodule" ]]; then
	cd ${ROOT} \
		&& git lfs install assets \
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
	cd ${RMM}
	rustup toolchain install $(cat ${ROOT}/rmm/rust-toolchain)
	rustup target add aarch64-unknown-none-softfloat
	rustup component add rust-src
)

pip3 install pre-commit
pre-commit install
