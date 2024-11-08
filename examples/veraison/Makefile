ROOT = $(shell git rev-parse --show-toplevel)
ISLET_RA := ${ROOT}/../remote-attestation
ROCLI = ${ISLET_RA}/tools/rocli
RSICTL = ${ISLET_RA}/tools/rsictl
OUT = ${ROOT}/out/shared
TARGET_X86_64 = x86_64-unknown-linux-gnu
TARGET_ARM = aarch64-unknown-linux-gnu

export PATH := ${ROOT}/assets/toolchain/aarch64-none-linux-gnu/bin:${PATH}

all: ${OUT} bin/rocli ${OUT}/bin/rsictl bin/rsictl bin/reliant-party ${OUT}/bin/realm-application ${OUT}/root-ca.crt

${OUT}:
	mkdir -p "${OUT}"

bin/rocli: ${ROCLI} ${ROCLI}/src
	cargo install --profile release --path "${ROCLI}" --root . --target ${TARGET_X86_64}

${OUT}/bin/rsictl: ${RSICTL} ${RSICTL}/src
	cargo install --profile release --path "${RSICTL}" --root "${OUT}" --target ${TARGET_ARM}

bin/rsictl: ${RSICTL} ${RSICTL}/src
	cargo install --profile release --path "${RSICTL}" --root . --target ${TARGET_X86_64}

bin/reliant-party: reliant-party reliant-party/src
	cargo install --profile release --path reliant-party --root . --target ${TARGET_X86_64}

${OUT}/bin/realm-application: realm-application realm-application/src
	cargo install --profile release --path realm-application --root "${OUT}" --target ${TARGET_ARM}

${OUT}/root-ca.crt: realm-application/root-ca.crt
	cp realm-application/root-ca.crt ${OUT}

clean:
	cargo clean --profile release --manifest-path "${ROCLI}"/Cargo.toml --target ${TARGET_X86_64}
	cargo clean --profile release --manifest-path "${RSICTL}"/Cargo.toml --target ${TARGET_ARM}
	cargo clean --profile release --manifest-path reliant-party/Cargo.toml --target ${TARGET_X86_64}
	cargo clean --profile release --manifest-path realm-application/Cargo.toml --target ${TARGET_ARM}
	rm -rf bin
	rm -rf "${OUT}"/bin
	rm -f "${OUT}"/root-ca.crt
