#!/bin/sh

cargo build
cp -f ../../../out/x86_64-unknown-linux-gnu/debug/cloak-two-way-tls ./
cp -f ../prebuilt/cvm1.crt ./
cp -f ../prebuilt/cvm1.key ./
cp -f ../prebuilt/chain1.crt ./
cp -f ../prebuilt/cvm2.crt ./
cp -f ../prebuilt/cvm2.key ./
cp -f ../prebuilt/chain2.crt ./
cp -f ../prebuilt/root.crt ./
cp -f ../prebuilt/root.key ./
