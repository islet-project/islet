#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/examples/cross-platform-e2ee
CERTIFIER=$ROOT/third-party/certifier
OPENSSL_INCLUDE=$ROOT/assets/openssl/include
PROTO_INCLUDE=$HERE/protobuf/src
THIRD_PARTY_ARM=$HERE/third-party-arm

EXAMPLE=$CERTIFIER/sample_apps/simple_app_under_islet
TARGET=$ROOT/out/shared/certifier-client

# Sync latest islet sdk
cd $ROOT/sdk
make sdk

mkdir -p $CERTIFIER/third_party/islet/lib
cp $ROOT/out/aarch64-unknown-linux-gnu/release/libislet_sdk.a $CERTIFIER/third_party/islet/lib

# Build example
cd $EXAMPLE
make -f islet_example_app.mak clean

LOCAL_LIB=$THIRD_PARTY_ARM \
	OPENSSL_INCLUDE=$OPENSSL_INCLUDE \
	PROTO_INCLUDE=$PROTO_INCLUDE \
	TARGET_MACHINE_TYPE=ARM64 \
	CC=aarch64-linux-gnu-g++ \
	LINK=aarch64-linux-gnu-g++ \
	AR=aarch64-linux-gnu-ar \
	make -f islet_example_app.mak

cp islet_example_app.exe $TARGET
