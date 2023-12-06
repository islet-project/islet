#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/examples/cross-platform-e2ee

CERTIFIER=$ROOT/third-party/certifier
EXAMPLE=$CERTIFIER/sample_apps/simple_app_under_islet

cd $EXAMPLE
rm -rf server client service provisioning
make -f islet_example_app.mak clean
