ROOT      = $(shell git rev-parse --show-toplevel)
LINUX     = $(ROOT)/third-party/realm-linux
TOOLCHAIN = $(ROOT)/assets/toolchain/aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-
TARGETDIR = $(ROOT)/out/realm
TARGET    = $(TARGETDIR)/linux.realm

all: config build

.PHONY: config
config:
	make -C $(LINUX) \
		defconfig ARCH=arm64 \
		KBUILD_DEFCONFIG=realm \
		CROSS_COMPILE=$(TOOLCHAIN)

.PHONY: build
build:
	make -C $(LINUX) \
		-j$(shell nproc) ARCH=arm64 \
		CROSS_COMPILE=$(TOOLCHAIN)

.PHONY: install
install:
	mkdir -p $(TARGETDIR)
	cp $(LINUX)/arch/arm64/boot/Image $(TARGET)

.PHONY: clean
clean: 
	make -C $(LINUX) clean
