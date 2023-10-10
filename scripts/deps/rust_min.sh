#!/usr/bin/env bash

ROOT=$(dirname -- "$0")/../..
MSRV=$(cat $ROOT/rust-toolchain)

if ! which rustup &>/dev/null; then
	wget --no-check-certificate https://sh.rustup.rs -O rustup-init.sh
	cat rustup-init.sh | sh -s -- -y --profile=minimal --default-toolchain $MSRV
	rm rustup-init.sh
	source $HOME/.cargo/env
fi

rustup toolchain install $MSRV
rustup target add aarch64-unknown-none-softfloat

rustc --version --verbose
