#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/examples/cross-platform-e2ee

CERTIFIER=$ROOT/third-party/certifier
OPENSSL=$ROOT/assets/openssl/lib
THIRD_PARTY_ARM=$HERE/third-party-arm

sudo apt install libgtest-dev libgflags-dev
sudo apt install autoconf automake libtool curl make g++ unzip

sudo apt install uuid-dev wget
sudo apt install openssl libssl-dev
sudo apt install g++-aarch64-linux-gnu

# Setup utilities
cd $HERE && git clone https://github.com/protocolbuffers/protobuf.git
cd protobuf && git checkout 21.x
git submodule update --init --recursive
./autogen.sh && ./configure
make -j$(nproc) && sudo make install
sudo ldconfig

cd $CERTIFIER/utilities
make -f cert_utility.mak clean
make -f cert_utility.mak
make -f policy_utilities.mak

# Setup certifier service
wget https://go.dev/dl/go1.18.4.linux-amd64.tar.gz
sudo rm -rf /usr/local/go && sudo tar -C /usr/local -xzf go1.18.4.linux-amd64.tar.gz
rm go1.18.4.linux-amd64.tar.gz

export PATH=$PATH:/usr/local/go/bin && export PATH=$PATH:$(go env GOPATH)/bin
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest

# Setup third-party libs for arm64
mkdir -p $THIRD_PARTY_ARM

cd $HERE/protobuf
make clean
./autogen.sh && ./configure --host=aarch64-linux-gnu --build=x86_64-linux-gnu \
	--enable-cross-compile --with-protoc=protoc
make
cp src/.libs/libprotobuf.a $THIRD_PARTY_ARM

cd $HERE && git clone https://github.com/gflags/gflags
cd $HERE/gflags
cmake -DCMAKE_TOOLCHAIN_FILE=$HERE/toolchain-arm.cmake
make
cp lib/libgflags.a $THIRD_PARTY_ARM

cd $HERE && git clone https://github.com/google/googletest
cd $HERE/googletest
cmake -DCMAKE_TOOLCHAIN_FILE=$HERE/toolchain-arm.cmake
make
cd $HERE/googletest
cp lib/libgtest.a $THIRD_PARTY_ARM

cp $OPENSSL/* $THIRD_PARTY_ARM
