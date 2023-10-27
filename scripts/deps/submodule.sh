#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)

cd $ROOT && git submodule update --init --depth 1 --recursive
