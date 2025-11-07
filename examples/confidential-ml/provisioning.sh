#!/bin/bash
# ref: https://github.com/vmware-research/certifier-framework-for-confidential-computing/blob/main/sample_apps/simple_app_under_islet/instructions.md

set -e

ROOT=$(git rev-parse --show-toplevel)

HERE=$ROOT/examples/confidential-ml
CERTIFIER_DATA=$HERE/certifier-data
CERTIFIER=$ROOT/third-party/certifier

EXAMPLE=$CERTIFIER/sample_apps/simple_app_under_islet
FVP_SHARED=$ROOT/out/shared

# Set your measurements
RUNTIME_MEASUREMENT=580bd77074f789f34841ea9920579ff29a59b9452b606f73811132b31c689da9
DEVICE_MEASUREMENT=84556646e9f83a30241d74181e906d5a23075a22370edef5db53882367c839f8
MODEL_PROVIDER_MEASUREMENT=84556646e9f83a30241d74181e906d5a23075a22370edef5db53882367c839f9

rm -rf $CERTIFIER_DATA
mkdir $CERTIFIER_DATA
cd $CERTIFIER_DATA
mkdir -p provisioning service runtime device model-provider
mkdir -p $FVP_SHARED/device

cd $CERTIFIER_DATA/provisioning

## Generate & embed a policy key
$CERTIFIER/utilities/cert_utility.exe \
	--operation=generate-policy-key-and-test-keys \
	--policy_key_output_file=policy_key_file.bin \
	--policy_cert_output_file=policy_cert_file.bin \
	--platform_key_output_file=platform_key_file.bin \
	--attest_key_output_file=attest_key_file.bin

$CERTIFIER/utilities/embed_policy_key.exe \
	--input=policy_cert_file.bin \
	--output=$CERTIFIER_DATA/policy_key.cc

cd $CERTIFIER_DATA/provisioning

$CERTIFIER/utilities/measurement_init.exe \
	--mrenclave=$RUNTIME_MEASUREMENT \
	--out_file=runtime.measurement

$CERTIFIER/utilities/measurement_init.exe \
	--mrenclave=$DEVICE_MEASUREMENT \
	--out_file=device.measurement

$CERTIFIER/utilities/measurement_init.exe \
	--mrenclave=$MODEL_PROVIDER_MEASUREMENT \
	--out_file=model_provider.measurement

cd $CERTIFIER_DATA/provisioning
cp -p policy_cert_file.bin cca_emulated_islet_key_cert.bin

$CERTIFIER/utilities/make_unary_vse_clause.exe \
	--key_subject=platform_key_file.bin \
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
	--measurement_subject=runtime.measurement \
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
	--measurement_subject=device.measurement \
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

$CERTIFIER/utilities/make_unary_vse_clause.exe \
        --key_subject="" \
        --measurement_subject=model_provider.measurement \
        --verb="is-trusted" \
        --output=ts4.bin

$CERTIFIER/utilities/make_indirect_vse_clause.exe \
        --key_subject=policy_key_file.bin \
        --verb="says" \
        --clause=ts4.bin \
        --output=vse_policy4.bin

$CERTIFIER/utilities/make_signed_claim_from_vse_clause.exe \
        --vse_file=vse_policy4.bin \
        --duration=9000 \
        --private_key_file=policy_key_file.bin \
        --output=signed_claim_4.bin

$CERTIFIER/utilities/package_claims.exe \
	--input=signed_claim_1.bin,signed_claim_2.bin,signed_claim_3.bin,signed_claim_4.bin \
	--output=policy.bin

$CERTIFIER/utilities/print_packaged_claims.exe --input=policy.bin

$CERTIFIER/utilities/make_unary_vse_clause.exe    \
      --key_subject=attest_key_file.bin                     \
      --verb="is-trusted-for-attestation"                   \
      --output=tsc1.bin

$CERTIFIER/utilities/make_indirect_vse_clause.exe    \
      --key_subject=platform_key_file.bin                      \
      --verb="says"                                            \
      --clause=tsc1.bin                                        \
      --output=vse_policy5.bin

$CERTIFIER/utilities/make_signed_claim_from_vse_clause.exe    \
      --vse_file=vse_policy5.bin                                        \
      --duration=9000                                                   \
      --private_key_file=platform_key_file.bin                          \
      --output=platform_attest_endorsement.bin

$CERTIFIER/utilities/print_signed_claim.exe --input=platform_attest_endorsement.bin

## Server & Client
cd $CERTIFIER_DATA/provisioning
cp -p ./* $CERTIFIER_DATA/runtime/
cp -p ./* $CERTIFIER_DATA/device/
cp -p ./* $CERTIFIER_DATA/model-provider/
cp -p ./* $FVP_SHARED/device/

## Service
cd $CERTIFIER_DATA/provisioning
cp -p policy_key_file.bin policy_cert_file.bin policy.bin $CERTIFIER_DATA/service

## Client script
#cp -p $HERE/run-client.sh $FVP_SHARED
