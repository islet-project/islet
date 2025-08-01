#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)

cd $ROOT && git submodule update --init --depth 1
echo $PWD $ROOT
cd $ROOT && git submodule foreach '
    if [ "$(basename $path)" != "qemu" ]; then
        git submodule update --init --depth 1 --recursive
    fi
    true'
