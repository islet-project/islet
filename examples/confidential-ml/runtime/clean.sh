#!/bin/sh

make -f ../common/build.mak clean EXE=runtime
rm -f certifier.pb.h certifier.pb.cc
