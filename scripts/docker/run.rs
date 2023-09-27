#!/usr/bin/env bash

set -e

ROOT=$(git rev-parse --show-toplevel)

sudo docker run -it -v $ROOT:/islet islet:latest
