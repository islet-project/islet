#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
HERE=${ROOT}/scripts

source ${HERE}/env.sh

(
    cd ${FASTMODEL}/Base_RevC_AEMvA_pkg/models/Linux64_GCC-6.4/
    ./FVP_Base_RevC-2xAEMvA  \
        -C bp.flashloader0.fname=${ROOT}/out/fip.bin \
        -C bp.secureflashloader.fname=${ROOT}/out/bl1.bin \
        -f ${HERE}/.config \
        -Q 1000 "$@"
)
