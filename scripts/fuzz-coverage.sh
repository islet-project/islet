#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

FUZZER=$1
TIMEOUT=$2

mkdir -p $ROOT/out

cd $ROOT/rmm/
COVERAGE=1 $ROOT/scripts/fuzz.sh $FUZZER -max_total_time=$TIMEOUT

rm -rf ../code-coverage
grcov . -s . --binary-path $ROOT/out/aarch64-unknown-linux-gnu/fuzz/ -t html --ignore tests/ -o ../code-coverage
