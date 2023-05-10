#!/bin/sh

make -f ../common/build.mak clean EXE=runtime
./clean_ckpt.sh
rm -rf data/
