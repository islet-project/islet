#!/bin/sh

RUSTFLAGS="-C target-feature=-crt-static" cargo build -r --target=aarch64-unknown-linux-gnu
cp -f /home/jinbum/ssd/github/islet/out/aarch64-unknown-linux-gnu/release/cvm_* /home/jinbum/ssd/github/islet/out/shared/
cp -f ../prebuilt/* /home/jinbum/ssd/github/islet/out/shared/
