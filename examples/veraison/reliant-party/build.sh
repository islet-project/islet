#!/bin/sh

cargo build
cp -f ./target/x86_64-unknown-linux-gnu/debug/reliant-party ../bin/
