#!/bin/sh

cp -f ../certifier-data/certifier.pb.cc .
make -f ../common/build.mak CC=g++ EXE=model_provider INC_PATH=/usr/local/include TENSOR_FLOW=OFF
rm -f certifier.pb.cc
