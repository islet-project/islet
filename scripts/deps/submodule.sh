#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
THIRD_PARTY=$ROOT/third-party
TF_RMM=$THIRD_PARTY/tf-rmm
KVM_UNIT_TESTS=$THIRD_PARTY/kvm-unit-tests

cd $TF_RMM && git submodule update --init
cd $KVM_UNIT_TESTS && git submodule update --init
