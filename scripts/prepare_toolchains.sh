#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

cd ${ROOT}/assets
if [ ! -d toolchains ]; then
    mkdir toolchains
fi

cd ${ROOT}/assets/toolchains
if [ ! -f gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz ]; then
    echo "gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz does NOT exist"
    wget https://developer.arm.com/-/media/Files/downloads/gnu-a/10.2-2020.11/binrel/gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz
    echo "Extracting gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz"
    tar -xf gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz
    ln -sf gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu aarch64

    echo "Creating symbolic links for aarch64/bin"
    cd ${ROOT}/toolchains/aarch64/bin
    check=`find -maxdepth 1 -type f `
    for file in $check
    do
	filename=`echo $file | cut -d '/' -f 2`
	linkname=`echo $file | cut -d '-' -f 1,3-`
	if [[ $linkname != "" ]];then
	    ln -sf $filename $linkname
	fi
    done
else
    echo "gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz already exists"
fi

cd ${ROOT}/assets/toolchains
if [ ! -f gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz ]; then
    echo "gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz does NOT exist"
    wget https://developer.arm.com/-/media/Files/downloads/gnu-a/10.2-2020.11/binrel/gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz
    echo "Extracting gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz"
    tar -xf gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz
    ln -sf gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf aarch32

    echo "Creating symbolic links for aarch32/bin"
    cd ${ROOT}/toolchains/aarch32/bin
    check=`find -maxdepth 1 -type f `
    for file in $check
    do
	filename=`echo $file | cut -d '/' -f 2`
	linkname=`echo $file | cut -d '-' -f 1,3-`
	if [[ $linkname != "" ]];then
	    ln -sf $filename $linkname
	fi
    done
fi

