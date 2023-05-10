#!/bin/bash

set -e

cd ..
cargo build
cd -
g++ attestation.cpp \
	-lislet_sdk \
	-L/data/islet/out/x86_64-unknown-linux-gnu/debug/ \
	-I ../include
LD_LIBRARY_PATH=/data/islet/out/x86_64-unknown-linux-gnu/debug/ ./a.out
