#!/usr/bin/env bash

ROOT=$(dirname -- "$0")/../..

cd $ROOT && git submodule update --init
if [ $? -ne 0 ]; then
	echo "Failed to sync assets."
	exit 1
fi
