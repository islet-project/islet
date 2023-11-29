ROOT = $(shell git rev-parse --show-toplevel)
ISLET_RA := ${ROOT}/../islet-ra
ROCLI = ${ISLET_RA}/tools/rocli
RSICTL = ${ISLET_RA}/tools/rsictl
OUT = ${ROOT}/out/shared
TARGET_X86 = x86_64-unknown-linux-gnu
TARGET_ARM = aarch64-unknown-linux-gnu

export PATH := ${ROOT}/assets/toolchain/aarch64-none-linux-gnu/bin:${PATH}

all: ${OUT} bin/rocli ${OUT}/bin/rsictl bin/reliant-party ${OUT}/bin/realm-application ${OUT}/root-ca.crt

${OUT}:
	mkdir -p "${OUT}"

bin/rocli: ${ROCLI} ${ROCLI}/src
	cargo install --profile release --path "${ROCLI}" --root . --target ${TARGET_X86}

${OUT}/bin/rsictl: ${RSICTL} ${RSICTL}/src
	cargo install --profile release --path "${RSICTL}" --root "${OUT}" --target ${TARGET_ARM}

bin/reliant-party: reliant-party reliant-party/src
	cargo install --profile release --path reliant-party --root . --target ${TARGET_X86}

${OUT}/bin/realm-application: realm-application realm-application/src
	cargo install --profile release --path realm-application --root "${OUT}" --target ${TARGET_ARM}

${OUT}/root-ca.crt:
	cp realm-application/root-ca.crt ${OUT}
