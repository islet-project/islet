#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
HERE=$ROOT/examples/confidential-ml
SERVICE_PROVISION_DIR=$HERE/certifier-data/service
HOST_IP=192.168.10.1

export PATH=$PATH:/usr/local/go/bin && export PATH=$PATH:$(go env GOPATH)/bin
export LD_LIBRARY_PATH=$CERTIFIER/certifier_service/teelib:$LD_LIBRARY_PATH
export LD_LIBRARY_PATH=$CERTIFIER/certifier_service/isletlib:$LD_LIBRARY_PATH
export LD_LIBRARY_PATH=$CERTIFIER/certifier_service/graminelib:$LD_LIBRARY_PATH
export LD_LIBRARY_PATH=$CERTIFIER/certifier_service/oelib:$LD_LIBRARY_PATH

cd $SERVICE_PROVISION_DIR
$CERTIFIER/certifier_service/simpleserver --policyFile=policy.bin --readPolicy=true --host=$HOST_IP -key_service_host=$HOST_IP
