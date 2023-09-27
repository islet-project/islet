#!/usr/bin/env bash

set -e

ROOT=$(git rev-parse --show-toplevel)

sudo docker build $ROOT -t islet
