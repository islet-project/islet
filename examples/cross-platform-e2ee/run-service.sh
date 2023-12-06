#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
EXAMPLE_DIR=$CERTIFIER/sample_apps/simple_app_under_islet

export PATH=$PATH:/usr/local/go/bin && export PATH=$PATH:$(go env GOPATH)/bin

cd $EXAMPLE_DIR/service
$CERTIFIER/certifier_service/simpleserver --policyFile=policy.bin --readPolicy=true --host=193.168.10.15 -key_service_host=193.168.10.15
