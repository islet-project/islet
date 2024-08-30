#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

mkdir -p $ROOT/out

cd $ROOT/rmm

# rmi::features::test::rmi_features
# rmi::gpt::test::rmi_granule_delegate_negative
# rmi::gpt::test::rmi_granule_delegate_positive
# rmi::gpt::test::rmi_granule_undelegate
# rmi::realm::test::rmi_realm_create_negative
# rmi::realm::test::rmi_realm_create_positive
# rmi::version::test::rmi_version
if [ $# -eq 0 ]; then
	MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo miri test \
		rmi::features::test::rmi_features \
		--target aarch64-unknown-linux-gnu -- --nocapture
	MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo miri test \
		rmi::version::test::rmi_version \
		--target aarch64-unknown-linux-gnu -- --nocapture
else
	MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo miri test $1 \
		--target aarch64-unknown-linux-gnu -- --nocapture
fi

if [ $? -ne 0 ]; then
	exit 1
fi
