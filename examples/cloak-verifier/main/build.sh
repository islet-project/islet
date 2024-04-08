#!/bin/sh

cargo build -r --target=x86_64-unknown-linux-gnu
mkdir /home/jinbum/ssd/github/islet/out/shared
cp -f /home/jinbum/ssd/github/islet/out/x86_64-unknown-linux-gnu/release/cvm_* /home/jinbum/ssd/github/islet/out/shared/
cp -f ../prebuilt/* /home/jinbum/ssd/github/islet/out/shared/
