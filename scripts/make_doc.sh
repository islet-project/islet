#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

(
	cd ${ROOT}/docs
	mdbook build

	cd ${ROOT}/rmm
	cargo doc --lib --no-deps -p monitor

	cd ..
	cp -R out/doc out/book/crates
)
