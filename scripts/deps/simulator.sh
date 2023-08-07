#!/usr/bin/env bash

set -e

ROOT=$(git rev-parse --show-toplevel)
FVP=$ROOT/third-party/fvp
URL=https://developer.arm.com/-/media/Files/downloads/ecosystem-models/FVP_Base_RevC-2xAEMvA_11.20_15_Linux64.tgz
TAR=FVP_Base_RevC-2xAEMvA_11.20_15_Linux64.tgz

if [ ! -d "$FVP" ]; then
	mkdir $FVP
	wget $URL
	tar xf $TAR -C $FVP
	rm $TAR
fi
