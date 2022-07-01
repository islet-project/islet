#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

# Download toolchains for cross-compilation
cd ${ROOT}
mkdir -p assets/toolchains
cd assets/toolchains
wget https://developer.arm.com/-/media/Files/downloads/gnu-a/10.2-2020.11/binrel/gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz
wget https://developer.arm.com/-/media/Files/downloads/gnu-a/10.2-2020.11/binrel/gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz
tar -xf gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz
tar -xf gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf.tar.xz
ln -sf gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu aarch64
ln -sf gcc-arm-10.2-2020.11-x86_64-arm-none-linux-gnueabihf aarch32

# Setup symbolic links to be used for cross-compilation
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
