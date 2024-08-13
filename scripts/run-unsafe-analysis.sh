#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
TOOL=$ROOT/third-party/utrace

cd $TOOL
CRATE_PATH=$ROOT/plat/fvp make call-trace >$ROOT/unsafe-call-trace.log
CRATE_PATH=$ROOT/plat/fvp make unsafe-list | tee $ROOT/unsafe-items-list.log
