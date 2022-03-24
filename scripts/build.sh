#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

mkdir -p ${ROOT}/out

(
	cd ${TF_A_TESTS}
	make CROSS_COMPILE=${CROSS_COMPILE} PLAT=fvp DEBUG=1
)

if [ $? -ne 0 ]; then
	echo "[!] ${TF_A_TESTS} build failed "
	exit 1
fi

if [ ! -f "${FIPTOOL}" ]; then
	cd ${TRUSTED_FIRMWARE_A}/tools/fiptool
	make
fi

if [ $? -ne 0 ]; then
	echo "[!] ${FIPTOOL} build failed "
	exit 1
fi

(
	cd ${TRUSTED_FIRMWARE_A}
	make CROSS_COMPILE=${CROSS_COMPILE} PLAT=fvp ENABLE_RME=1 FVP_HW_CONFIG_DTS=fdts/fvp-base-gicv3-psci-1t.dts DEBUG=1 all
	cp build/fvp/debug/bl1.bin ${ROOT}/out/.
)

if [ $? -ne 0 ]; then
	echo "[!] ${TRUSTED_FIRMWARE_A} build failed "
	exit 1
fi

(
	cd ${VM_IMAGE}
	make CROSS_COMPILE=${CROSS_COMPILE} PLAT=fvp DEBUG=1 tftf
	cp ${VM_IMAGE}/build/fvp/debug/tftf.bin ${ROOT}/out/vm-image.bin

	if [ $? -ne 0 ]; then
		exit 1
	fi

	${FIPTOOL} create \
		--fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_fw_config.dtb \
		--tb-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_tb_fw_config.dtb \
		--soc-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_soc_fw_config.dtb \
		--nt-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_nt_fw_config.dtb \
		--hw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp-base-gicv3-psci-1t.dtb \
		--tb-fw ${TRUSTED_FIRMWARE_A}/build/fvp/debug/bl2.bin \
		--soc-fw ${TRUSTED_FIRMWARE_A}/build/fvp/debug/bl31.bin \
		--rmm-fw ${TRUSTED_FIRMWARE_A}/build/fvp/debug/rmm.bin \
		--nt-fw ${ROOT}/out/vm-image.bin \
		${ROOT}/out/fip-vm-image.bin
)

if [ $? -ne 0 ]; then
	echo "[!] ${VM_IMAGE} build failed "
	exit 1
fi

(
	cd ${RMM}
	cargo build --release
	${CROSS_COMPILE}objcopy -O binary ${ROOT}/out/aarch64-unknown-none-softfloat/release/rmm ${ROOT}/out/aarch64-unknown-none-softfloat/release/rmm.bin
)

if [ $? -ne 0 ]; then
	echo "[!] ${RMM} build failed "
	exit 1
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
