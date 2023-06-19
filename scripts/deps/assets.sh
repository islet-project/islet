#!/usr/bin/env bash

set -e

ROOT=$(git rev-parse --show-toplevel)

cd $ROOT && git submodule update --init assets
