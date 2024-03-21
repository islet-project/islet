#!/bin/sh

./set-realm-ip.sh
date 120512002023
cp -f /shared/cloak-verifier /
cp -f /shared/root-ca.crt /
cp -f /shared/priv_key.serde.arm64 /
