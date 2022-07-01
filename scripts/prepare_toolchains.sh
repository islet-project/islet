#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

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
