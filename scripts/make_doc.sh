#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

mv ${ROOT}/rust-toolchain ${ROOT}/rust-toolchain.bak
cargo install mdbook mdbook-plantuml
mv ${ROOT}/rust-toolchain.bak ${ROOT}/rust-toolchain

(
	cd ${ROOT}/docs
	mdbook build

	cd ${ROOT}
	cargo doc --lib --no-deps -p realm-management-monitor
	mv out/aarch64-unknown-none-softfloat/doc out/.
	cp -R out/doc out/book/crates
)
