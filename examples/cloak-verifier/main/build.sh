#!/bin/sh

cargo build -r --target=x86_64-unknown-linux-gnu
cp -f /home/jinbum/ssd/github/islet/out/x86_64-unknown-linux-gnu/release/cloak-verifier /home/jinbum/ssd/github/islet/out/shared/
cp -f ../prebuilt/* /home/jinbum/ssd/github/islet/out/shared/
