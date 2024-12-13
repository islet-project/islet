ROOT = $(shell git rev-parse --show-toplevel)

LINUX_RSI_DIR = ${ROOT}/third-party/linux-rsi
export KERNEL_DIR := ${ROOT}/third-party/realm-linux
export OUTPUT_DIR := ${ROOT}/out/shared
export PATH := ${ROOT}/assets/toolchain/aarch64-none-linux-gnu/bin:${PATH}
export CROSS_COMPILE := aarch64-none-linux-gnu-
export ARCH := arm64

CMDLINE = https://github.com/islet-project/rsictl.git
CONFIG_ARM = --config target.aarch64-unknown-linux-gnu.linker='"aarch64-none-linux-gnu-gcc"' \
             --config target.aarch64-unknown-linux-gnu.rustflags='[ "-C", "target-feature=+crt-static" ]'
TARGET_ARM = aarch64-unknown-linux-gnu

all: module cmdline

module: ${HEADERS}
	cd ${LINUX_RSI_DIR} && make

cmdline:
	cargo install ${CONFIG_ARM} --git "${CMDLINE}" --root "${OUTPUT_DIR}" --target ${TARGET_ARM}

clean:
	cd ${LINUX_RSI_DIR} && make clean
