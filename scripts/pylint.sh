#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/scripts

# TODO: Add fvp-cca
pylint --disable=C0103,C0114,C0116,W0621,W1401,W1514 $HERE/unsafe-analyzer
