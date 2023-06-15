#!/usr/bin/env bash

set -e

ROOT=$(dirname -- "$0")/../..

cd $ROOT && git submodule update --init --recursive
