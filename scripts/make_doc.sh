#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)

# Link READMEs
cd ${ROOT}/doc
ln -sf ../README.md intro.md
cd ${ROOT}/doc/components
ln -sf ../../sdk/README.md sdk.md
ln -sf ../../cli/README.md cli.md
ln -sf ../../rmm/README.md rmm.md
cd ${ROOT}/doc/usecases
ln -sf ../../examples/confidential-ml/README.md confidential-ml.md
ln -sf ../../examples/cross-platform-e2ee/README.md cross-platform-e2ee.md
ln -sf ../../examples/veraison/README.md remote-attestation.md

# RMM crate
rm -rf ${ROOT}/doc/plat-doc
cargo doc --lib --no-deps -p islet_rmm
cp -R ${ROOT}/out/aarch64-unknown-none-softfloat/doc ${ROOT}/doc/plat-doc

# SDK crate
rm -rf ${ROOT}/doc/app-doc
cd ${ROOT}/sdk
cargo doc --lib --no-deps -p islet_sdk
cp -R ${ROOT}/out/x86_64-unknown-linux-gnu/doc ${ROOT}/doc/app-doc

rm -rf ${ROOT}/book
cd ${ROOT}/doc
mdbook build

rm -rf ${ROOT}/docs
mv ${ROOT}/out/book ${ROOT}/docs

# Clear
rm -rf ${ROOT}/doc/plat-doc
rm -rf ${ROOT}/doc/app-doc
