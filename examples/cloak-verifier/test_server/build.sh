#!/bin/sh

cargo build
mkdir ../../../out/shared
cp -f ../../../out/x86_64-unknown-linux-gnu/debug/cloak-test-server ../../../out/shared/
