#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

cargo install mdbook mdbook-plantuml

(
	cd ${ROOT}/docs
	mdbook build

	cd ${ROOT}
	cargo doc --lib --no-deps -p realm-management-monitor
	mv out/aarch64-unknown-none-softfloat/doc out/.
	cp -R out/doc out/book/crates
)
