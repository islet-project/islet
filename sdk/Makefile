ROOT = $(shell git rev-parse --show-toplevel)
HERE = $(ROOT)/sdk

GPP_ARM64 = ../assets/toolchain/aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-g++
SDK_INCLUDE= -I$(HERE)/include
SDK_LDFLAGS= -L$(ROOT)/out/aarch64-unknown-linux-gnu/release -lislet_sdk

SHARED_DIR = $(ROOT)/out/shared
OPENSSL_ROOT = $(ROOT)/assets/openssl
OPENSSL_INCLUDE = $(OPENSSL_ROOT)/include
OPENSSL_ARM64_FLAG = OPENSSL_STATIC=1 OPENSSL_LIB_DIR=$(OPENSSL_ROOT)/lib-arm64 OPENSSL_INCLUDE_DIR=$(OPENSSL_INCLUDE)
OPENSSL_X64_FLAG = OPENSSL_STATIC=1 OPENSSL_LIB_DIR=$(OPENSSL_ROOT)/lib-x64 OPENSSL_INCLUDE_DIR=$(OPENSSL_INCLUDE)

CFLAGS += $(SDK_INCLUDE)
LDFLAGS += $(SDK_LDFLAGS)

all: simulated sdk

sdk:
	@$(OPENSSL_ARM64_FLAG) cargo build -r --target=aarch64-unknown-linux-gnu

sdk-example:
	@$(OPENSSL_ARM64_FLAG) cargo build -r --example attest_n_seal --target=aarch64-unknown-linux-gnu

sdk-example-c: examples/c_api/attest_seal_test.cc
	@$(OPENSSL_ARM64_FLAG) RUSTFLAGS="-C target-feature=-crt-static" cargo build -r --target=aarch64-unknown-linux-gnu
	@$(GPP_ARM64) $< $(CFLAGS) $(LDFLAGS) -o $@

test:
	@cargo test -- --nocapture

simulated:
	@$(OPENSSL_X64_FLAG) cargo build -r --target=x86_64-unknown-linux-gnu

run-simulated:
	@cargo run -r --example attest_n_seal --target=x86_64-unknown-linux-gnu

run-simulated-c:
	@$(OPENSSL_X64_FLAG) cargo build -r --target=x86_64-unknown-linux-gnu
	@g++ examples/c_api/attest_seal_test.cc -lislet_sdk \
		$(CFLAGS) \
		-L$(ROOT)/out/x86_64-unknown-linux-gnu/release \
		-lssl -lcrypto -lpthread -ldl \
		-o sdk-example-c
	@LD_LIBRARY_PATH=$(ROOT)/out/x86_64-unknown-linux-gnu/release/ ./sdk-example-c

fvp: sdk sdk-example
	@mkdir -p $(SHARED_DIR)
	@install -m755 $(ROOT)/out/aarch64-unknown-linux-gnu/release/examples/attest_n_seal $(SHARED_DIR)/sdk-example
	@$(MAKE) sdk-example-c
	@cp -p $(ROOT)/out/aarch64-unknown-linux-gnu/release/libislet_sdk.a  $(SHARED_DIR)
	@install -m755 sdk-example-c $(SHARED_DIR)

clean:
	@cd $(HERE); cargo clean; rm -rf sdk-example-c
