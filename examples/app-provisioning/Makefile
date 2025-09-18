ROOT = $(shell git rev-parse --show-toplevel)
ROCLI = https://github.com/islet-project/rocli.git
RSICTL = https://github.com/islet-project/rsictl.git
OUT = ${ROOT}/out/shared
CONFIG_ARM = --config target.aarch64-unknown-linux-gnu.linker='"aarch64-none-linux-gnu-gcc"' --config target.aarch64-unknown-linux-gnu.rustflags='[ "-C", "target-feature=+crt-static" ]'
TARGET_X86_64 = x86_64-unknown-linux-gnu
TARGET_ARM = aarch64-unknown-linux-gnu

export PATH := ${ROOT}/assets/toolchain/aarch64-none-linux-gnu/bin:${PATH}

all: bin/rocli bin/rsictl

bin/rocli:
	cargo install --profile release --git "${ROCLI}" --root . --target ${TARGET_X86_64}

bin/rsictl:
	cargo install --profile release --git "${RSICTL}" --root . --target ${TARGET_X86_64}

clean:
	rm -rf bin
