#!/usr/bin/env python3

import subprocess
import argparse
import subprocess

from config import * 

def run_qemu(islet_dir, aosp_dir, initramfs_path, kernel_path, ramdisk_path, tf_a_path):
    subprocess.run(["./scripts/run_qemu.sh", islet_dir, aosp_dir, initramfs_path, kernel_path, ramdisk_path, tf_a_path])

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="QEMU CCA script")
    parser.add_argument("-aosp", action="store_true", help="Run AOSP with CCA supported host kernel on QEMU")
    parser.add_argument("-aosp-prebuilt", action="store_true", help="Use prebuilt binaries")
    args = parser.parse_args()

    if args.aosp:
        print("Run AOSP with CCA supported host kernel on QEMU")
        run_qemu(ROOT, AOSP_DIR,
                 AOSP_KERNEL_INITRAMFS_PATH, AOSP_KERNEL_IMAGE_PATH,
                 PREBUILT_QEMU_RME_RAMDISK, PREBUILT_QEMU_RME_TF_A)
    elif args.aosp_prebuilt:
        print("Run AOSP with CCA supported prebuilt host kernel on QEMU")
        run_qemu(ROOT, AOSP_DIR,
                 PREBUILT_QEMU_RME_HOST_INITRAMFS, PREBUILT_QEMU_RME_HOST_IMAGE,
                 PREBUILT_QEMU_RME_RAMDISK, PREBUILT_QEMU_RME_TF_A)
    else:
        print("There is no option to run. Exiting...")
