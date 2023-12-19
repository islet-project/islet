#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
EXAMPLE_DIR=$CERTIFIER/sample_apps/simple_app_under_islet

TARGET=certifier-server

# Sync latest islet sdk
cd $ROOT/sdk
make simulated

mkdir -p $CERTIFIER/third_party/islet/lib
cp $ROOT/out/x86_64-unknown-linux-gnu/release/libislet_sdk.a $CERTIFIER/third_party/islet/lib

# Build server app
cd $EXAMPLE_DIR
make -f islet_example_app.mak clean
OPENSSL_INCLUDE=$ROOT/assets/openssl/include \
	OPENSSL_LIB=$ROOT/assets/openssl/lib-x64 \
	make -f islet_example_app.mak
cp islet_example_app.exe $TARGET
