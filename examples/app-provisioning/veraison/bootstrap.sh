#!/bin/bash

set -exuo pipefail
shopt -s expand_aliases

ROOT_DIR=$(git rev-parse --show-toplevel)
VERAISON_DIR="$ROOT_DIR/examples/app-provisioning/veraison"

ROCLI_REPO="https://github.com/islet-project/remote-attestation"
ROCLI_DIR="$VERAISON_DIR/remote-attestation"
ROCLI_SUB_DIR="$VERAISON_DIR/remote-attestation/tools/rocli"

SERVICES_REPO="https://github.com/veraison/services.git"
SERVICES_DIR="$VERAISON_DIR/services"
DOCKER_DIR="$SERVICES_DIR/deployments/docker"
REVISION="736e119"

go install github.com/veraison/ear/arc@e895c1e

if [ -d "$ROCLI_DIR" ]
then
	rm -rf "$VERAISON_DIR/bin"
    rm -rf "$ROCLI_DIR";
fi

git clone "$ROCLI_REPO" "$ROCLI_DIR"
echo "[workspace]" >> "${ROCLI_SUB_DIR}"/Cargo.toml
cargo install --profile release --path "${ROCLI_SUB_DIR}" --root . --target x86_64-unknown-linux-gnu

if [ -d "$SERVICES_DIR" ]
then
    make  really-clean -C "$DOCKER_DIR"
    rm -rf "$SERVICES_DIR";
fi

git clone "$SERVICES_REPO" "$SERVICES_DIR"
pushd "$SERVICES_DIR"
git checkout -b freeze "$REVISION"
popd

for patch in "$VERAISON_DIR/veraison-no-auth-patch" ; do
    cat "$patch" | (cd "$SERVICES_DIR" && git apply);
done

make -C "$DOCKER_DIR"

source "$DOCKER_DIR/env.bash"

veraison start

echo "Waiting for the services to be available:"
until pocli create ARM_CCA "$VERAISON_DIR/provision/accept-all.rego" -i >/dev/null; do
    sleep 2
done
