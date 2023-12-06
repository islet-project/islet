#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
FVP=$ROOT/scripts/fvp-cca

$FVP --normal-world=linux-net --realm=linux --hes --build-only
$FVP --normal-world=linux-net --realm=linux --hes --run-only &

sleep 10

telnet localhost 5000
