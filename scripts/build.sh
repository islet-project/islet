#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

(
	cd ${TF_A_TESTS}
	make CROSS_COMPILE=${GCC} PLAT=fvp DEBUG=1
)

(
	cd ${TRUSTED_FIRMWARE_A}
	make CROSS_COMPILE=${GCC} PLAT=fvp ENABLE_RME=1 FVP_HW_CONFIG_DTS=fdts/fvp-base-gicv3-psci-1t.dts DEBUG=1 BL33=${TF_A_TESTS}/build/fvp/debug/tftf.bin all fip
)
