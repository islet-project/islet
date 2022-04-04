#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

mkdir -p ${ROOT}/out

function fn_prepare_fiptool()
{
	if [ ! -f "${FIPTOOL}" ]; then
		cd ${TRUSTED_FIRMWARE_A}/tools/fiptool
		make
	fi
}

function fn_build()
{
	### TODO Refactor this to use ./build
	(
		cd ${RMM}
		cargo build --release
		${CROSS_COMPILE}objcopy -O binary ${ROOT}/out/aarch64-unknown-none-softfloat/release/fvp ${ROOT}/out/aarch64-unknown-none-softfloat/release/rmm.bin
	)

	if [ $? -ne 0 ]; then
		echo "[!] ${RMM} build failed "
		exit 1
	fi

	fn_prepare_fiptool
	if [ ! -f "${FIPTOOL}" ]; then
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
		cd ${TF_A_TESTS}
		make CROSS_COMPILE=${CROSS_COMPILE} PLAT=fvp DEBUG=1

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
			${ROOT}/out/fip-tf-a-tests.bin
	)

	if [ $? -ne 0 ]; then
		echo "[!] ${TF_A_TESTS} build failed "
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

	${FIPTOOL} create \
		--fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_fw_config.dtb \
		--tb-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_tb_fw_config.dtb \
		--soc-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_soc_fw_config.dtb \
		--nt-fw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp_nt_fw_config.dtb \
		--hw-config ${TRUSTED_FIRMWARE_A}/build/fvp/debug/fdts/fvp-base-gicv3-psci-1t.dtb \
		--tb-fw ${TRUSTED_FIRMWARE_A}/build/fvp/debug/bl2.bin \
		--soc-fw ${TRUSTED_FIRMWARE_A}/build/fvp/debug/bl31.bin \
		--rmm-fw ${ROOT}/out/aarch64-unknown-none-softfloat/release/rmm.bin \
		--nt-fw ${ROOT}/out/FVP_AARCH64_EFI.fd \
		${ROOT}/out/fip.bin

	cd ${BUILD_SCRIPT} \
		&& make -j$(nproc) -f fvp.mk boot-img
}

function fn_build_thirdparty()
{
	cd ${BUILD_SCRIPT} \
		&& make -j$(nproc) -f fvp.mk grub edk2

	cp ${THIRD_PARTY}/edk2-platforms/Build/ArmVExpress-FVP-AArch64/RELEASE_GCC49/FV/FVP_AARCH64_EFI.fd ${ROOT}/out/.
}

function fn_prepare_prebuilt_thirdparty
{
	cp ${PREBUILT}/FVP_AARCH64_EFI.fd ${ROOT}/out/.
	cp ${PREBUILT}/bootaa64.efi ${ROOT}/out/.
}

function fn_build_linux()
{
	cd ${BUILD_SCRIPT} \
		&& make -j$(nproc) -f fvp.mk linux

	cp ${ROOT}/linux/arch/arm64/boot/Image ${ROOT}/out/.
	cp ${ROOT}/linux/arch/arm64/boot/dts/arm/fvp-base-revc.dtb ${ROOT}/out/.
}

function fn_prepare_prebuilt_linux
{
	cp ${PREBUILT}/Image ${ROOT}/out/.
	cp ${PREBUILT}/fvp-base-revc.dtb ${ROOT}/out/.
}

function fn_usage()
{
	echo "./${SCRIPT_NAME} [OPTIONS]"
	cat <<EOF
no option:
    Do unit-test and print the results
options:
    --unit-test  Get test results as a JUnit xml file to out/test-result.xml
    --coverage   Measure coverage tests and get results in out/coverage
EOF
}

BUILD_THIRDPARTY=false
BUILD_LINUX=false

while [ $# -gt 0 ]; do
	case "$1" in
		--third-party)
			BUILD_THIRDPARTY=true
			;;
		--linux)
			BUILD_LINUX=true
			;;
		--help | -h)
			fn_usage
			exit
			;;
	esac
	shift
done

if ${BUILD_THIRDPARTY}; then
	fn_build_thirdparty
	if [ $? -ne 0 ]; then
		echo "[!] thirdparty build failed "
		exit 1
	fi
else
	fn_prepare_prebuilt_thirdparty
fi

if ${BUILD_LINUX}; then
	fn_build_linux
	if [ $? -ne 0 ]; then
		echo "[!] linux build failed "
		exit 1
	fi
else
	fn_prepare_prebuilt_linux
fi

fn_build
