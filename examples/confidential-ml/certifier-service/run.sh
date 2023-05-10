#!/bin/sh

ARCH=$1
cp -f ../certifier-data/* .
./simpleserver.${ARCH} --policyFile=policy.bin --readPolicy=true --host="0.0.0.0"
