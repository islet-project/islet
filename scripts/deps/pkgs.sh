#!/bin/bash

set -e

apt update && apt -y install sudo

sudo apt install -y -qq --no-install-recommends \
	gcc-multilib cmake libssl-dev \
	binutils python3-pip \
	device-tree-compiler xterm fakeroot mtools fdisk cpio \
	dosfstools e2fsprogs \
	libxml-libxml-perl \
	jq lcov graphviz inkscape \
	flex bison \
	bzip2 \
	srecord \
	git wget make vim

pip3 install toml
