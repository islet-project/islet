#!/bin/bash

set -e

apt update && apt -y install sudo

sudo apt install -y -qq --no-install-recommends \
	gcc-multilib g++-multilib cmake libssl-dev \
	binutils python3-pip \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	flex bison \
	bzip2 \
	srecord \
	git wget make vim bc pkg-config \
	bridge-utils \
	pylint

pip3 install toml
