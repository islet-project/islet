#!/bin/sh

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
HERE=$ROOT/examples/confidential-ml/runtime
TF=$ROOT/examples/confidential-ml/tensorflow_src

protoc --proto_path=$CERTIFIER/certifier_service/certprotos --cpp_out=. $CERTIFIER/certifier_service/certprotos/certifier.proto
cp -f $CERTIFIER/include/certifier.pb.h $CERTIFIER/include/certifier.pb.h.orig
cp -f certifier.pb.h $CERTIFIER/include/

make -f $HERE/../common/build.mak CC=g++ EXE=runtime INC_PATH=${TF} CERTIFIER=${CERTIFIER} LIB_PATH=$HERE/../tflite_libs TENSOR_FLOW=ON

