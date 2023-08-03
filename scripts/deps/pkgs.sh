#!/bin/bash

set -e

sudo apt update
sudo apt install -y -qq --no-install-recommends \
	gcc-multilib cmake libssl-dev \
	binutils python3-pip \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	flex bison \
	srecord

pip3 install toml
