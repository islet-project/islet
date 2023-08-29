ROOT = $(shell git rev-parse --show-toplevel)
HERE = $(ROOT)/sdk

GPP_ARM64 = ../assets/toolchain/aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-g++
SDK_INCLUDE= -I$(HERE)/include
SDK_LDFLAGS= -L$(ROOT)/out/aarch64-unknown-linux-gnu/release -lislet_sdk

SHARED_DIR = $(ROOT)/out/shared

CFLAGS += $(SDK_INCLUDE)
LDFLAGS += $(SDK_LDFLAGS)

all: simulated sdk

sdk:
	@cargo build -r --target=aarch64-unknown-linux-gnu

sdk-example:
	cargo build -r --example aarch64 --target=aarch64-unknown-linux-gnu

sdk-example-c: examples/c_api/attest_seal_test.cc
	@RUSTFLAGS="-C target-feature=-crt-static" cargo build -r --target=aarch64-unknown-linux-gnu
	@$(GPP_ARM64) $< $(CFLAGS) $(LDFLAGS) -o $@

test:
	@cargo test -- --nocapture

simulated:
	@cargo build -r --target=x86_64-unknown-linux-gnu

run-simulated:
	@cargo run -r --example simulated --target=x86_64-unknown-linux-gnu

run-simulated-c:
	@cargo build -r --target=x86_64-unknown-linux-gnu
	@g++ examples/c_api/attest_seal_test.cc -lislet_sdk \
		$(CFLAGS) \
		-L$(ROOT)/out/x86_64-unknown-linux-gnu/release \
		-o sdk-example-c
	@LD_LIBRARY_PATH=$(ROOT)/out/x86_64-unknown-linux-gnu/release/ ./sdk-example-c

fvp: sdk sdk-example
	@mkdir -p $(SHARED_DIR)
	@install -m755 $(ROOT)/out/aarch64-unknown-linux-gnu/release/examples/aarch64 $(SHARED_DIR)/sdk-example
	@$(MAKE) sdk-example-c
	@cp -p $(ROOT)/out/aarch64-unknown-linux-gnu/release/libislet_sdk.a  $(SHARED_DIR)
	@install -m755 sdk-example-c $(SHARED_DIR)

clean:
	@cd $(HERE); cargo clean; rm -rf sdk-example-c
