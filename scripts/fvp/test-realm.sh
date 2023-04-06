#!/bin/sh

set -e

LKVM="/shared/lkvm"
ARGS="--realm --irqchip=gicv3 --console=serial --network mode=none --nodefaults"

cd kvm-unit-tests

if [ "$1" == "attest" ]; then
	LKVM=$LKVM $LKVM run $ARGS -c 1 -m 24 -k ./realm-attest.flat -p "attest"
	LKVM=$LKVM $LKVM run $ARGS -c 1 -m 24 -k ./realm-attest.flat -p "extend"
	LKVM=$LKVM $LKVM run $ARGS -c 1 -m 24 -k ./realm-attest.flat -p "extend_and_attest"
	LKVM=$LKVM $LKVM run $ARGS -c 1 -m 24 -k ./realm-attest.flat -p "measurement"
else
	LKVM=$LKVM ./run-realm-tests
fi
