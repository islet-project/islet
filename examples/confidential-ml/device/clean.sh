#!/bin/sh

make -f ../common/build.mak clean EXE=device
rm -f certifier.pb.h certifier.pb.cc
