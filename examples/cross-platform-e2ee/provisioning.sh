#!/bin/bash
# ref: https://github.com/vmware-research/certifier-framework-for-confidential-computing/blob/main/sample_apps/simple_app_under_islet/instructions.md

set -e

ROOT=$(git rev-parse --show-toplevel)

HERE=$ROOT/examples/cross-platform-e2ee
CERTIFIER=$ROOT/third-party/certifier

EXAMPLE=$CERTIFIER/sample_apps/simple_app_under_islet
FVP_SHARED=$ROOT/out/shared

# Set your measurements
SERVER_MEASUREMENT=580bd77074f789f34841ea9920579ff29a59b9452b606f73811132b31c689da9
CLIENT_MEASUREMENT=491cf94bdb951308672a839776359d6ac22808bad2d318226ef0ea2979693e2e

cd $EXAMPLE
mkdir -p provisioning service server client
mkdir -p $FVP_SHARED/client

cd $EXAMPLE/provisioning

## Generate & embed a policy key
$CERTIFIER/utilities/cert_utility.exe \
	--operation=generate-policy-key-and-test-keys \
	--policy_key_output_file=policy_key_file.bin \
	--policy_cert_output_file=policy_cert_file.bin \
	--platform_key_output_file=platform_key_file.bin \
	--attest_key_output_file=attest_key_file.bin

$CERTIFIER/utilities/embed_policy_key.exe \
	--input=policy_cert_file.bin \
	--output=../policy_key.cc

cd $EXAMPLE/provisioning

$CERTIFIER/utilities/measurement_init.exe \
	--mrenclave=$SERVER_MEASUREMENT \
	--out_file=server.measurement

$CERTIFIER/utilities/measurement_init.exe \
	--mrenclave=$CLIENT_MEASUREMENT \
	--out_file=client.measurement

cd $EXAMPLE/provisioning
cp -p policy_cert_file.bin cca_emulated_islet_key_cert.bin

$CERTIFIER/utilities/make_unary_vse_clause.exe \
	--cert-subject=cca_emulated_islet_key_cert.bin \
	--verb="is-trusted-for-attestation" \
	--output=ts1.bin

$CERTIFIER/utilities/make_indirect_vse_clause.exe \
	--key_subject=policy_key_file.bin \
	--verb="says" \
	--clause=ts1.bin \
	--output=vse_policy1.bin

$CERTIFIER/utilities/make_signed_claim_from_vse_clause.exe \
	--vse_file=vse_policy1.bin \
	--duration=9000 \
	--private_key_file=policy_key_file.bin \
	--output=signed_claim_1.bin

$CERTIFIER/utilities/make_unary_vse_clause.exe \
	--key_subject="" \
	--measurement_subject=server.measurement \
	--verb="is-trusted" \
	--output=ts2.bin

$CERTIFIER/utilities/make_indirect_vse_clause.exe \
	--key_subject=policy_key_file.bin \
	--verb="says" \
	--clause=ts2.bin \
	--output=vse_policy2.bin

$CERTIFIER/utilities/make_signed_claim_from_vse_clause.exe \
	--vse_file=vse_policy2.bin \
	--duration=9000 \
	--private_key_file=policy_key_file.bin \
	--output=signed_claim_2.bin

$CERTIFIER/utilities/make_unary_vse_clause.exe \
	--key_subject="" \
	--measurement_subject=client.measurement \
	--verb="is-trusted" \
	--output=ts3.bin

$CERTIFIER/utilities/make_indirect_vse_clause.exe \
	--key_subject=policy_key_file.bin \
	--verb="says" \
	--clause=ts3.bin \
	--output=vse_policy3.bin

$CERTIFIER/utilities/make_signed_claim_from_vse_clause.exe \
	--vse_file=vse_policy3.bin \
	--duration=9000 \
	--private_key_file=policy_key_file.bin \
	--output=signed_claim_3.bin

$CERTIFIER/utilities/package_claims.exe \
	--input=signed_claim_1.bin,signed_claim_2.bin,signed_claim_3.bin \
	--output=policy.bin

$CERTIFIER/utilities/print_packaged_claims.exe --input=policy.bin

## Server & Client
cd $EXAMPLE/provisioning
cp -p ./* $EXAMPLE/server
cp -p ./* $FVP_SHARED/client

## Service
cd $EXAMPLE/provisioning
cp -p policy_key_file.bin policy_cert_file.bin policy.bin $EXAMPLE/service

## Client script
cp -p $HERE/run-client.sh $FVP_SHARED
