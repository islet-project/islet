#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

(
	cd  ${ROOT}
	git submodule update --init --recursive
)

sudo apt install -y -qq device-tree-compiler xterm
