# wasm-realm uses kernel of linux-realm
include ../linux/Makefile

WASMFS = $(ROOT)/assets/prebuilt/rootfs/rootfs-realm-wasm.cpio.gz
KERNEL = $(TARGET)

# Run on host
.PHONY: run
run: config build install
	qemu-system-aarch64 \
		-kernel $(KERNEL) \
		-initrd $(WASMFS) \
		-cpu cortex-a57 \
		-smp 2 \
		-M virt \
		-m 256M \
		-nographic
