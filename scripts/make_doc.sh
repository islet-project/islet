#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
BOOK=${ROOT}/out/book

# RMM crate
rm -rf ${ROOT}/doc/plat-doc
cd ${ROOT}
cargo doc --lib --no-deps -p islet_rmm
cp -R ${ROOT}/out/aarch64-unknown-none-softfloat/doc ${ROOT}/doc/plat-doc

# SDK crate
rm -rf ${ROOT}/doc/app-doc
cd ${ROOT}/sdk
cargo doc --lib --no-deps -p islet_sdk
cp -R ${ROOT}/out/x86_64-unknown-linux-gnu/doc ${ROOT}/doc/app-doc

# HES crate
rm -rf ${ROOT}/doc/hes-doc
cd ${ROOT}/hes
cargo doc --lib --no-deps -p islet-hes
cp -R ${ROOT}/out/x86_64-unknown-linux-gnu/doc ${ROOT}/doc/hes-doc

rm -rf ${BOOK}
cd ${ROOT}/doc
mdbook build

rm -rf ${ROOT}/docs
mv ${ROOT}/out/book ${ROOT}/docs

# Clear
rm -rf ${ROOT}/doc/plat-doc
rm -rf ${ROOT}/doc/app-doc
rm -rf ${ROOT}/doc/hes-doc
