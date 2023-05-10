#!/bin/sh

make -f ../common/build.mak clean EXE=device
./clean_ckpt.sh
rm -rf data/
