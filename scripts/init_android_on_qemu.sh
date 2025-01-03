#!/bin/bash

PARENT_DIR="third-party/android_on_qemu"

AOSP_VER="aosp-15.0.0_r8"
AOSP_DIR="$PARENT_DIR/$AOSP_VER"
AOSP_URL="https://android.googlesource.com/platform/manifest"
AOSP_BRANCH="android-15.0.0_r8"

ANDROID_KERNEL_VER="android15-6.6"
ANDROID_KERNEL_DIR="$PARENT_DIR/$ANDROID_KERNEL_VER"
ANDROID_KERNEL_URL="https://github.com/islet-project/3rd-android-kernel.git"
ANDROID_KERNEL_BRANCH="common-android15-6.6/cca-host/rmm-v1.0-eac5"

CUR_SCRIPT_DIR=$(dirname "$(realpath "$0")")
ISLET_DIR=$(dirname "$CUR_SCRIPT_DIR")

INITRAMFS_PATH="$ISLET_DIR/$ANDROID_KERNEL_DIR/out/virtual_device_aarch64/dist/initramfs.img"
KERNEL_PATH="$ISLET_DIR/$ANDROID_KERNEL_DIR/out/virtual_device_aarch64/dist/Image"

function install_required_packages()
{
	if [ -z "$(which repo)" ]; then
		sudo apt-get install repo
	fi
}

function build_aosp()
{
	cd "$ISLET_DIR" || exit

	# Create aosp directory and download AOSP sources
	if [ -d "$AOSP_DIR" ]; then
		echo "$AOSP_DIR already exists."

		echo "Changing directory to $AOSP_DIR..."
		cd $AOSP_DIR || exit 1 # if cd failed, exit with error code
	else
		echo "Creating directory $AOSP_DIR..."
		mkdir -p $AOSP_DIR

		echo "Changing directory to $AOSP_DIR..."
		cd $AOSP_DIR || exit 2 # if cd failed, exit with error code

		echo "Downloading AOSP sources..."
		repo init --partial-clone -b $AOSP_BRANCH -u $AOSP_URL

		if ! repo sync -c -j8; then
			echo "ERROR: Download AOSP failed"
			exit 3
		fi
	fi

	if [ -f "out/host/linux-x86/bin/launch_cvd" ]; then
		echo "launch_cvd is exists. Skip building AOSP"
		return
	fi

	echo "Setting up build environment..."
	source build/envsetup.sh

	echo "Choosing a target..."
	lunch aosp_cf_arm64_only_phone-trunk_staging-userdebug

	echo "Building AOSP..."
	if ! m; then
		echo "ERROR: AOSP Build failed"
		exit 4
	fi

	echo "Go back to $ISLET_DIR..."
	cd "$ISLET_DIR" || exit 5 # if cd failed, exit with error code
}

function build_android_kernel()
{
	cd "$ISLET_DIR" || exit

	if [ ! -d "$ANDROID_KERNEL_DIR" ]; then
		echo "Creating directory $ANDROID_KERNEL_DIR..."
		mkdir -p "$ANDROID_KERNEL_DIR"
	fi

	echo "Changing directory to $ANDROID_KERNEL_DIR..."
	cd $ANDROID_KERNEL_DIR || exit 1

	if [ ! -d "common" ]; then
		echo "Downloading Android Kernel sources..."
		repo init --partial-clone -u https://android.googlesource.com/kernel/manifest -b common-$ANDROID_KERNEL_VER
		if ! repo sync; then
			echo "ERROR: Download Android Kernel failed"
			exit 2
		fi
		echo "Replace common with cca patched kernel sources..."
		mv common backup_common
		git clone $ANDROID_KERNEL_URL -b $ANDROID_KERNEL_BRANCH --depth 1 --single-branch common
	fi

	if [ -f "$INITRAMFS_PATH" ] && [ -f "$KERNEL_PATH" ]; then
		echo "Build is alread done. Skip building Android Kernel"
		return
	fi

	# Build
	echo "Building Android Kernel..."
	if ! tools/bazel run //common-modules/virtual-device:virtual_device_aarch64_dist_internal; then
		echo "ERROR: Android Kernel Build failed"
		exit 4
	fi

	echo "Check built images..."
	realpath $INITRAMFS_PATH
	realpath $KERNEL_PATH

	echo "Go back to $ISLET_DIR..."
	cd $ISLET_DIR || exit 5
}

install_required_packages

build_aosp

build_android_kernel
