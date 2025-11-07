#!/bin/sh

make -f ../common/build.mak clean EXE=model_provider
rm -f certifier.pb.h certifier.pb.cc
