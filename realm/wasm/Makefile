# wasm-realm uses kernel of linux-realm
include ../linux/Makefile

WASMFS = $(ROOT)/assets/prebuilt/qemu/guest/wasm-realm-initrd.cpio.gz
KERNEL = $(TARGET)

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
