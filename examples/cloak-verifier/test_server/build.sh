#!/bin/sh

cargo build
cp -f ../../../out/x86_64-unknown-linux-gnu/debug/cloak-test-server ../../../out/shared/
