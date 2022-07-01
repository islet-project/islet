#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

cd ${ROOT}
mkdir -p assets/fastmodel
cd assets/fastmodel
wget https://developer.arm.com/-/media/Files/downloads/ecosystem-models/FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz
tar -xzf FVP_Base_RevC-2xAEMvA_11.18_16_Linux64.tgz
