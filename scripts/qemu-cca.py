#!/usr/bin/env python3

import subprocess
import argparse
import subprocess

from config import ROOT, AOSP_DIR, AOSP_KERNEL_INITRAMFS_PATH, AOSP_KERNEL_IMAGE_PATH, AOSP_KERNEL_SRC_DIR

def run_qemu(islet_dir, aosp_dir, initramfs_path, kernel_path, kernel_src_dir):
    subprocess.run(["./scripts/run_qemu.sh", islet_dir, aosp_dir, initramfs_path, kernel_path, kernel_src_dir])

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="QEMU CCA script")
    parser.add_argument("-aosp", action="store_true", help="Run AOSP with CCA supported host kernel on QEMU")
    args = parser.parse_args()

    if args.aosp:
        print("Run AOSP with CCA supported host kernel on QEMU")
        run_qemu(ROOT, AOSP_DIR, AOSP_KERNEL_INITRAMFS_PATH, AOSP_KERNEL_IMAGE_PATH, AOSP_KERNEL_SRC_DIR)
    else:
        print("There is no option to run. Exiting...")
