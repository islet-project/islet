#!/bin/bash

set -e

apt-get update && apt -y install sudo

sudo apt-get update

sudo apt-get install -y -qq --no-install-recommends --fix-missing \
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
	inotify-tools \
	pylint

python3 -m venv venv
source venv/bin/activate
pip3 install toml
deactivate
