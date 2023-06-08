ROOT = $(shell git rev-parse --show-toplevel)
HERE = $(ROOT)/cli
TARGET = islet

GPP_ARM64 = ../assets/toolchain/aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-g++
SDK_INCLUDE= -I$(HERE)/include
SDK_LDFLAGS= -L$(ROOT)/out/aarch64-unknown-linux-gnu/release -lislet_sdk

SHARED_DIR = $(ROOT)/out/shared

all: x86_64 aarch64

aarch64:
	@mkdir -p  ${SHARED_DIR}
	@cargo build -r --target=aarch64-unknown-linux-gnu
	@install -m755 ${HERE}/target/aarch64-unknown-linux-gnu/release/islet_cli ${SHARED_DIR}/islet

x86_64:
	@mkdir -p  ${SHARED_DIR}
	@cargo build -r --target=x86_64-unknown-linux-gnu
	@install -m755 ${HERE}/target/x86_64-unknown-linux-gnu/release/islet_cli ${HERE}/islet

clean:
	@cd $(HERE); cargo clean; rm -rf sdk-example-c
