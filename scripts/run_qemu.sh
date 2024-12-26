#!/bin/bash

ISLET_DIR=$1
AOSP_DIR=$2
INITRAMFS_PATH=$3
KERNEL_PATH=$4

function run_qemu()
{
	cd "$ISLET_DIR" || exit
	# Go to the aosp source directory which you were built before
	echo "Changing directory to $AOSP_DIR to run qemu..."
	cd $AOSP_DIR || exit 1

	# Setup environment & select the target again
	echo "Setting up build environment..."
	. build/envsetup.sh
	lunch aosp_cf_arm64_only_phone-trunk_staging-userdebug

	# Run cuttlefish with cca support linux -> after start kernel, there is no logs..
	echo "Running Cuttlefish based by QEMU..."
	launch_cvd -vm_manager qemu_cli -enable_host_bluetooth false -report_anonymous_usage_stats=n \
		-initramfs_path $INITRAMFS_PATH \
		-kernel_path $KERNEL_PATH
}

run_qemu
