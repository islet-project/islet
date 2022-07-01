#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

cd ${ROOT}/assets
if [ ! -d fastmodel ]; then
    mkdir fastmodel
fi

cd ${ROOT}/assets/fastmodel
if [ ! -f FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz ]; then
    echo "FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz does NOT exist"
    wget https://developer.arm.com/-/media/Files/downloads/ecosystem-models/FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz
    echo "Extracting FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz"
    tar -xzf FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz
else
    echo "FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz already exists"
fi
