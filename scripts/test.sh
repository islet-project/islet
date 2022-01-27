#!/bin/bash

cargo test --lib --target x86_64-unknown-linux-gnu -- --test-threads=1
