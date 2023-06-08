ROOT = $(shell git rev-parse --show-toplevel)
HERE = ${ROOT}/realm/linux-rsi

export KERNEL_DIR := ${ROOT}/third-party/realm-linux
export SHARED_DIR := ${ROOT}/out/shared
export PATH := ${ROOT}/assets/toolchain/aarch64-none-linux-gnu/bin:${PATH}
export CROSS_COMPILE := aarch64-none-linux-gnu-
export ARCH := arm64

CMDLINE = ${ROOT}/cli
HEADERS = rsi.h

obj-m += rsi.o

all: module cmdline

module: ${HEADERS}
	make -C ${KERNEL_DIR} M=${HERE} modules
	cp rsi.ko ${SHARED_DIR}

cmdline:
	cd ${CMDLINE}; make aarch64

clean:
	make -C ${KERNEL_DIR} M=${HERE} clean
	cd ${CMDLINE}; cargo clean
