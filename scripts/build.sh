#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

(
	cd ${TF_A_TESTS}
	make CROSS_COMPILE=${CROSS_COMPILE} PLAT=fvp DEBUG=1
)

(
	cd ${TRUSTED_FIRMWARE_A}
	make CROSS_COMPILE=${CROSS_COMPILE} PLAT=fvp ENABLE_RME=1 FVP_HW_CONFIG_DTS=fdts/fvp-base-gicv3-psci-1t.dts DEBUG=1 all
	cp build/fvp/debug/bl1.bin ${ROOT}/out/.
)

(
	cd ${RMM}
	cargo build --release
	${CROSS_COMPILE}objcopy -O binary ${ROOT}/out/aarch64-unknown-none-softfloat/release/rmm ${ROOT}/out/aarch64-unknown-none-softfloat/release/rmm.bin
)

if [ ! -f "${FIPTOOL}" ]; then
	cd ${TRUSTED_FIRMWARE_A}/tools/fiptool
	make
fi

#Make fip.bin
${FIPTOOL} create \
	--fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_fw_config.dtb \
	--tb-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_tb_fw_config.dtb \
	--soc-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_soc_fw_config.dtb \
	--nt-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_nt_fw_config.dtb \
	--hw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp-base-gicv3-psci-1t.dtb \
	--tb-fw ${TRUSTED_FIRMWARE_A}/build/fvp/debug/bl2.bin \
	--soc-fw ${TRUSTED_FIRMWARE_A}/build/fvp/debug/bl31.bin \
	--rmm-fw ${ROOT}/out/aarch64-unknown-none-softfloat/release/rmm.bin \
	--nt-fw ${TF_A_TESTS}/build/fvp/debug/tftf.bin \
	${ROOT}/out/fip.bin
