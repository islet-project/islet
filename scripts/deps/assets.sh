#!/usr/bin/env bash

set -e

ROOT=$(git rev-parse --show-toplevel)

cd $ROOT && git submodule update --init assets
git submodule update --init --depth 1 $ROOT/third-party/ciborium
git submodule update --init --depth 1 $ROOT/third-party/coset
