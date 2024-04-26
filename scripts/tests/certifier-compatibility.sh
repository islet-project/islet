#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier

apt-get update && apt -y install sudo

sudo apt-get update

sudo apt-get install -y -qq --no-install-recommends --fix-missing \
	clang-format-11 libgtest-dev libgflags-dev \
	openssl libssl-dev protobuf-compiler protoc-gen-go golang-go cmake

git submodule update --init --depth 1 $CERTIFIER

cd $CERTIFIER
./CI/scripts/test.sh test-ISLET-SDK-shim_test
./CI/scripts/test.sh test-run_example-simple_app_under_islet-using-shim
