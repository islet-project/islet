#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

cd ${ROOT}/assets/prebuilt/qemu
if [ ! -f qemu-system-aarch64 ]; then
    echo "qemu-system-aarch64.zip does NOT exist"
    echo "Unzipping qemu-system-aarch64.zip"
    unzip qemu-system-aarch64.zip
else
    echo "qemu-system-aarch64.zip already exists"
fi
