#!/bin/sh

./set-realm-ip.sh
date 120512002024
cp -f /shared/cvm_all /
cp -f /shared/cvm_app /
cp -f /shared/cvm_gw /
cp -f /shared/*.crt /
cp -f /shared/*.key /
cp -f /shared/priv_key.serde.arm64 /
