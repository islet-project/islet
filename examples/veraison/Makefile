ROOT = $(shell git rev-parse --show-toplevel)
ROCLI = https://github.com/islet-project/rocli.git
RSICTL = https://github.com/islet-project/rsictl.git
OUT = ${ROOT}/out/shared
CONFIG_ARM = --config target.aarch64-unknown-linux-gnu.linker='"aarch64-none-linux-gnu-gcc"' --config target.aarch64-unknown-linux-gnu.rustflags='[ "-C", "target-feature=+crt-static" ]'
TARGET_X86_64 = x86_64-unknown-linux-gnu
TARGET_ARM = aarch64-unknown-linux-gnu

export PATH := ${ROOT}/assets/toolchain/aarch64-none-linux-gnu/bin:${PATH}

all: ${OUT} bin/rocli ${OUT}/bin/rsictl bin/rsictl bin/reliant-party ${OUT}/bin/realm-application ${OUT}/root-ca.crt

${OUT}:
	mkdir -p "${OUT}"

bin/rocli:
	cargo install --profile release --git "${ROCLI}" --root . --target ${TARGET_X86_64}

${OUT}/bin/rsictl:
	cargo install ${CONFIG_ARM} --profile release --git "${RSICTL}" --root "${OUT}" --target ${TARGET_ARM}

bin/rsictl:
	cargo install --profile release --git "${RSICTL}" --root . --target ${TARGET_X86_64}

bin/reliant-party: reliant-party reliant-party/src
	cargo install --profile release --path reliant-party --root . --target ${TARGET_X86_64}

${OUT}/bin/realm-application: realm-application realm-application/src
	cargo install ${CONFIG_ARM} --profile release --path realm-application --root "${OUT}" --target ${TARGET_ARM}

${OUT}/root-ca.crt: realm-application/root-ca.crt
	cp realm-application/root-ca.crt ${OUT}

clean:
	cargo clean --profile release --manifest-path reliant-party/Cargo.toml --target ${TARGET_X86_64}
	cargo clean --profile release --manifest-path realm-application/Cargo.toml --target ${TARGET_ARM}
	rm -rf bin
	rm -rf "${OUT}"/bin
	rm -f "${OUT}"/root-ca.crt
