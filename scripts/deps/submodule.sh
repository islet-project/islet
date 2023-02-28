#!/usr/bin/env bash

ROOT=$(dirname -- "$0")/../..

cd $ROOT/third-party/tf-rmm && git submodule update --init
if [ $? -ne 0 ]; then
	echo "Failed to sync submodule for third-party."
	exit 1
fi
