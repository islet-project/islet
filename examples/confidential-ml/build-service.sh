#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier

export PATH=$PATH:/usr/local/go/bin && export PATH=$PATH:$(go env GOPATH)/bin

# Sync latest islet sdk
cd $ROOT/sdk
make simulated

mkdir -p $CERTIFIER/third_party/islet/lib
cp $ROOT/out/x86_64-unknown-linux-gnu/release/libislet_sdk.a $CERTIFIER/third_party/islet/lib

cd $CERTIFIER/certifier_service/certprotos
protoc --go_opt=paths=source_relative \
	--go_out=. \
	--go_opt=M=certifier.proto \
	./certifier.proto

cd $CERTIFIER/certifier_service/oelib
make dummy

cd $CERTIFIER/certifier_service/graminelib
make dummy

cd $CERTIFIER/certifier_service/isletlib
make

cd $CERTIFIER/certifier_service/teelib
make

cd $CERTIFIER/certifier_service

go build simpleserver.go
