#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
HERE=${ROOT}/scripts

source ${HERE}/env.sh

(
	cd ${FASTMODEL}/Base_RevC_AEMvA_pkg/models/Linux64_GCC-6.4/
	case "$1" in
		--vm-image-test)
			shift
			./FVP_Base_RevC-2xAEMvA \
				-C bp.flashloader0.fname=${ROOT}/out/fip-vm-image.bin \
				-C bp.secureflashloader.fname=${ROOT}/out/bl1.bin \
				-f ${HERE}/.config \
				-Q 1000 "$@"
			;;
		*)
			./FVP_Base_RevC-2xAEMvA \
				-C bp.flashloader0.fname=${ROOT}/out/fip.bin \
				-C bp.secureflashloader.fname=${ROOT}/out/bl1.bin \
				--data="${ROOT}/out/vm-image.bin"@0x8806c000 \
				-f ${HERE}/.config \
				-Q 1000 "$@"
			;;
	esac
)
