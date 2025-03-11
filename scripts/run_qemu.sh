#!/bin/bash

ISLET_DIR=$1
AOSP_DIR=$2
INITRAMFS_PATH=$3
KERNEL_PATH=$4
AOSP_KERNEL_SRC_DIR_PATH=$5
AOSP_RME_RAMDISK_PATH="$(realpath $AOSP_KERNEL_SRC_DIR_PATH/aosp_rme_ramdisk.img)"
# This flash.bin is from linaro's repository.
# For more information, see the following link:
# https://linaro.atlassian.net/wiki/spaces/QEMU/pages/29596450823/Manual+build+instructions+for+TF-RMM+TF-A+and+host+EDK2+for+QEMU-virt
TF_A_TF_RMM_IMAGE_PATH="$(realpath $AOSP_KERNEL_SRC_DIR_PATH/flash.bin)"
KERNEL_CMDLINE="androidboot.hypervisor.vm.supported=1 vmw_vsock_virtio_transport_common.virtio_transport_max_vsock_pkt_buf_size=16384 console=ttynull stack_depot_disable=on cgroup_disable=pressure kasan.stacktrace=off bootconfig  printk.devkmsg=on audit=1 panic=-1 8250.nr_uarts=1 cma=0 firmware_class.path=/vendor/etc/ loop.max_part=7 init=/init bootconfig  console=hvc0 earlycon=pl011,mmio32,0x9000000 "

KERNEL_CMDLINE+=$AOSP_RME_RAMDISK_PATH



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
		--memory_mb 8192 \
		-initramfs_path $INITRAMFS_PATH \
		-kernel_path $KERNEL_PATH \
		-extra_kernel_cmdline $KERNEL_CMDLINE \
		-bootloader $TF_A_TF_RMM_IMAGE_PATH
}

run_qemu
