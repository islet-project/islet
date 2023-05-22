mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
current_dir := $(patsubst %/,%,$(dir $(mkfile_path)))
up_dir      := $(dir $(current_dir))

export KERNEL_DIR := ${up_dir}/fvp-cca-scripts/4.linux-cca-realm
export SHARED_DIR := ${up_dir}/fvp-cca-scripts/out/shared_dir
export PATH := ${up_dir}/fvp-cca-scripts/toolchains/arm-gnu-toolchain-11.3.rel1-x86_64-aarch64-none-linux-gnu/bin:${PATH}
export CROSS_COMPILE := aarch64-none-linux-gnu-
export ARCH := arm64

CMDLINE=rsictl
HEADERS=rsi.h

obj-m += rsi.o

all: module cmdline

module: ${HEADERS}
	make -C ${KERNEL_DIR} M=$(PWD) modules
	cp rsi.ko ${SHARED_DIR}

cmdline:
	cd ${CMDLINE}; cargo build -r --target aarch64-unknown-linux-gnu
	install -m755 ${CMDLINE}/target/aarch64-unknown-linux-gnu/release/${CMDLINE} ${SHARED_DIR}

clean:
	make -C ${KERNEL_DIR} M=$(PWD) clean
	cd ${CMDLINE}; cargo clean
