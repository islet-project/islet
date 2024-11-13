#!/bin/bash

set -exuo pipefail
shopt -s expand_aliases

ROOT_DIR=$(git rev-parse --show-toplevel)
SERVICES_REPO="https://github.com/veraison/services.git"
SERVICES_DIR="$ROOT_DIR/examples/veraison/services"
DOCKER_DIR="$SERVICES_DIR/deployments/docker"
REVISION="736e119"

if [ -d "$SERVICES_DIR" ]
then
    make  really-clean -C "$DOCKER_DIR"
    rm -rf "$SERVICES_DIR";
fi

go install github.com/veraison/ear/arc@latest

git clone --depth=1 "$SERVICES_REPO" "$SERVICES_DIR"
pushd "$SERVICES_DIR"
git checkout -b freeze "$REVISION"
popd

for patch in "./veraison-no-auth-patch" ; do
    cat "$patch" | (cd "$SERVICES_DIR" && git apply);
done

make -C "$DOCKER_DIR"

source "$DOCKER_DIR/env.bash"

veraison start
sleep 15
pocli create ARM_CCA accept-all.rego -i
