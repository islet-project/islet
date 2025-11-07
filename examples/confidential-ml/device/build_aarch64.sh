#!/bin/sh

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
HERE=$ROOT/examples/confidential-ml/device
TF=$ROOT/examples/confidential-ml/tensorflow_src
OPENSSL_INCLUDE=$ROOT/assets/openssl/include
PROTO_INCLUDE=$HERE/../protobuf/src
THIRD_PARTY_ARM=$HERE/../third-party-arm
SHARED=$ROOT/out/shared/device
TARGET=$ROOT/out/shared/device/device.exe

protoc --proto_path=$CERTIFIER/certifier_service/certprotos --cpp_out=. $CERTIFIER/certifier_service/certprotos/certifier.proto
cp -f $CERTIFIER/include/certifier.pb.h $CERTIFIER/include/certifier.pb.h.orig
cp -f certifier.pb.h $CERTIFIER/include/

make -f $HERE/../common/build.mak CC=aarch64-linux-gnu-g++ EXE=device CERTIFIER=${CERTIFIER} INC_PATH=${OPENSSL_INCLUDE} INC_PATH2=${PROTO_INCLUDE} LIB_PATH=$THIRD_PARTY_ARM TENSOR_FLOW=OFF

cp -f device.exe $TARGET
date +"%m%d%H%M%Y" > $SHARED/host.time
cp -f run_aarch64.sh $SHARED/
cp -f /usr/aarch64-linux-gnu/lib/libc.so.6 $SHARED/
cp -f /usr/aarch64-linux-gnu/lib/libstdc++.so.6 $SHARED/
