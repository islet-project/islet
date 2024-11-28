ROOT = $(shell git rev-parse --show-toplevel)
HERE = ${ROOT}/realm/linux-rsi

export KERNEL_DIR := ${ROOT}/third-party/realm-linux
export SHARED_DIR := ${ROOT}/out/shared
export PATH := ${ROOT}/assets/toolchain/aarch64-none-linux-gnu/bin:${PATH}
export CROSS_COMPILE := aarch64-none-linux-gnu-
export ARCH := arm64

CMDLINE = https://github.com/islet-project/rsictl.git
HEADERS = rsi.h
CONFIG_ARM = --config target.aarch64-unknown-linux-gnu.linker='"aarch64-none-linux-gnu-gcc"' \
             --config target.aarch64-unknown-linux-gnu.rustflags='[ "-C", "target-feature=+crt-static" ]'
TARGET_ARM = aarch64-unknown-linux-gnu

obj-m += rsi.o

all: module cmdline

module: ${HEADERS}
	make -C ${KERNEL_DIR} M=${HERE} modules
	cp rsi.ko ${SHARED_DIR}

cmdline:
	cargo install ${CONFIG_ARM} --git "${CMDLINE}" --root "${SHARED_DIR}" --target ${TARGET_ARM}

clean:
	make -C ${KERNEL_DIR} M=${HERE} clean
	cd ${CMDLINE}; cargo clean
