#!/bin/bash

CUR_SCRIPT_DIR=$(dirname "$(realpath "$0")")
ISLET_DIR=$(dirname "$CUR_SCRIPT_DIR")

QEMU_RME_DIR="$ISLET_DIR/third-party/android_on_qemu"
QEMU_SRC_DIR="$QEMU_RME_DIR/qemu"

AOSP_VER="aosp-15.0.0_r8"
AOSP_DIR="$QEMU_RME_DIR/$AOSP_VER"
AOSP_URL="https://android.googlesource.com/platform/manifest"
AOSP_BRANCH="android-15.0.0_r8"

ANDROID_KERNEL_VER="android16-6.12"
ANDROID_KERNEL_DIR="$QEMU_RME_DIR/$ANDROID_KERNEL_VER"
ANDROID_KERNEL_URL="https://github.com/islet-project/3rd-android-kernel.git"
ANDROID_KERNEL_BUILD_ENVIRONMENT_BRANCH="common-android16-6.12/cca-host/build/v5"
ANDROID_KERNEL_SOURCE_BRANCH="android16-6.12/cca-host/v5"
ANDROID_KERNEL_BUILD_TARGET="//common-modules/virtual-device:virtual_device_aarch64_dist"

INITRAMFS_PATH="$ANDROID_KERNEL_DIR/out/virtual_device_aarch64/dist/initramfs.img"
KERNEL_PATH="$ANDROID_KERNEL_DIR/out/virtual_device_aarch64/dist/Image"

QEMU_REPO="https://gitlab.com/qemu-project/qemu.git"
QEMU_TAG="v9.1.2"

function install_required_packages() {
	if [ -z "$(which repo)" ]; then
		sudo apt-get install repo
	fi
}

function build_aosp() {
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

function build_android_kernel() {
	cd "$ISLET_DIR" || exit

	if [ ! -d "$ANDROID_KERNEL_DIR" ]; then
		echo "Creating directory $ANDROID_KERNEL_DIR..."
		git clone $ANDROID_KERNEL_URL -b $ANDROID_KERNEL_BUILD_ENVIRONMENT_BRANCH --single-branch $ANDROID_KERNEL_DIR
	fi

	echo "Changing directory to $ANDROID_KERNEL_DIR..."
	cd $ANDROID_KERNEL_DIR || exit 1

	if [ ! -d "common" ]; then
		echo "unzip repo.zip to run 'repo sync'"
		if ! unzip repo.zip; then
			echo "ERROR: unzip repo.zip failed"
			exit 2
		fi

		echo "Downloading Android Kernel Build Modules..."
		if ! repo sync; then
			echo "ERROR: Failed to download Android Kernel Build Modules..."
			exit 3
		fi
	fi

	if [ ! -d "backup_common" ]; then
		echo "Replace common with cca patched kernel sources..."
		mv common backup_common
		if ! git clone $ANDROID_KERNEL_URL -b $ANDROID_KERNEL_SOURCE_BRANCH --depth 1 --single-branch common; then
			echo "ERROR: Failed to Download Android Kernel Source failed"
			mv backup_common common
			exit 4
		fi
	fi

	if [ -f "$INITRAMFS_PATH" ] && [ -f "$KERNEL_PATH" ]; then
		echo "Build is alread done. Skip building Android Kernel"
		return
	fi

	# Build
	echo "Building Android Kernel..."
	if ! tools/bazel run $ANDROID_KERNEL_BUILD_TARGET; then
		echo "ERROR: Android Kernel Build failed"
		exit 5
	fi

	echo "Check built images..."
	realpath $INITRAMFS_PATH
	realpath $KERNEL_PATH

	echo "Go back to $ISLET_DIR..."
	cd $ISLET_DIR || exit 6
}

function clone_qemu_source() {
	cd "$QEMU_RME_DIR" || exit 1

	if [ ! -d $QEMU_SRC_DIR ]; then
		echo "Cloning QEMU repository with tag $QEMU_TAG..."
		git clone --single-branch -b "$QEMU_TAG" --depth 1 "$QEMU_REPO" qemu
	fi
	cd qemu || exit 1

	# check the tag	hash
	TAG_HASH=$(git rev-list -n 1 "$QEMU_TAG")
	if [ -z "$TAG_HASH" ]; then
		echo "Error: Tag $QEMU_TAG not found"
		exit 1
	fi
}

function build_qemu() {
	cd "$QEMU_SRC_DIR" || exit 1

	if [ ! -d "build" ]; then
		echo "Configuring QEMU..."
		./configure --target-list=aarch64-softmmu --enable-slirp --disable-docs

		echo "Building QEMU..."
		make -j$(nproc)
	else
		echo "QEMU is already built. Skip building QEMU"
	fi
}

install_required_packages

clone_qemu_source
build_qemu

build_aosp

build_android_kernel
