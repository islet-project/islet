#!/bin/bash

set -e

sudo apt update
sudo apt install -y -qq --no-install-recommends \
	binutils python3-pip \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	flex bison

pip3 install toml
