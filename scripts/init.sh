#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

cd ${ROOT} && git submodule update --init --recursive

sudo apt install -y -qq --no-install-recommends \
	device-tree-compiler xterm fakeroot mtools \
	libxml-libxml-perl \
	jq lcov graphviz \
	openjdk-11-jre

rustup default nightly && rustup update
cargo install \
	mdbook mdbook-plantuml \
	cargo2junit \
	cargo-tarpaulin

rustup component add rustfmt

(
	cd ${RMM}
	rustup toolchain install $(cat ${ROOT}/rmm/rust-toolchain)
	rustup target add aarch64-unknown-none-softfloat
	rustup component add rust-src
)
