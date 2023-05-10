#!/bin/sh

cp -f ../certifier-data/certifier.pb.cc .
make -f ../common/build.mak CC=g++ EXE=device INC_PATH=/usr/local/include TENSOR_FLOW=ON
rm -f certifier.pb.cc
