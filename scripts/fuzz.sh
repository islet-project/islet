#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

mkdir -p $ROOT/out

cd $ROOT/rmm/fuzz

if [ $? -ne 0 ]; then
	exit 1
fi

export RUSTFLAGS="-C passes=sancov-module \
	-C llvm-args=-sanitizer-coverage-level=3 \
	-C llvm-args=-sanitizer-coverage-inline-8bit-counters \
	-C llvm-args=-sanitizer-coverage-trace-compares \
	--cfg fuzzing \
	-A warnings"

# Used by fuzz-coverage.sh to enable coverage mode
if [ "$COVERAGE" == "1" ]; then
	export RUSTFLAGS="$RUSTFLAGS -C instrument-coverage"
fi

if [ "$(uname --machine)" == "aarch64" ]; then
	# Note: ASAN does not work in QEMU userspace emulation
	# Hence, it is included only in this case.

	export RUSTFLAGS="$RUSTFLAGS -Z sanitizer=address"
	cargo run --profile fuzz --bin $1 -- ${@:2}
else
	cargo build --profile fuzz --bin $1

	if [ $? -ne 0 ]; then
		exit 1
	fi

	if ! which qemu-aarch64 &>/dev/null; then
		sudo apt-get update
		sudo apt-get install -y -qq --no-install-recommends qemu-user
	fi

	qemu-aarch64 \
		-E "LD_LIBRARY_PATH=../../assets/toolchain/aarch64-none-linux-gnu/aarch64-none-linux-gnu/lib64/:../../assets/toolchain/aarch64-none-linux-gnu/aarch64-none-linux-gnu/libc/lib64/" \
		-L ../../assets/toolchain/aarch64-none-linux-gnu/aarch64-none-linux-gnu/libc/ \
		../../out/aarch64-unknown-linux-gnu/fuzz/$1 -- ${@:2}
fi
