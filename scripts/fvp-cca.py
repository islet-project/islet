#!/usr/bin/env python3

import argparse
import os
import subprocess

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "out")
CONFIG = os.path.join(ROOT, "scripts/.config")

RMM = os.path.join(ROOT, "rmm/board/fvp")
TF_A_TESTS = os.path.join(ROOT, "tf-a-tests")
TRUSTED_FIRMWARE_A = os.path.join(ROOT, "trusted-firmware-a")
VM_IMAGE = os.path.join(ROOT, "vm-image")

CROSS_COMPILE = os.path.join(ROOT, "assets/toolchains/aarch64/bin/aarch64-none-linux-gnu-")
FASTMODEL = os.path.join(ROOT, "assets/fastmodel/Base_RevC_AEMvA_pkg/models/Linux64_GCC-6.4")
FIPTOOL = os.path.join(TRUSTED_FIRMWARE_A, "tools/fiptool")

os.makedirs(OUT, exist_ok=True)

def run(cmd, cwd):
    p = subprocess.run(cmd, cwd=cwd,
                       stderr=subprocess.STDOUT,
                       stdout=subprocess.PIPE,
                       universal_newlines=True)
    if p.returncode != 0:
        print("[!] Failed to run: %s @ %s" % (cmd, cwd))
        print(p.stdout)
        exit(1)

def make(srcdir, extra=None):
    args = ["make"]
    if extra:
        args += extra
    run(args, cwd=srcdir)

def prepare_tftf():
    global CROSS_COMPILE
    global OUT
    global TF_A_TESTS

    srcdir = TF_A_TESTS
    outbin = os.path.join(TF_A_TESTS, "build/fvp/debug/tftf.bin")

    args = [
        "CROSS_COMPILE=%s" % CROSS_COMPILE,
        "PLAT=fvp",
        "DEBUG=1"
    ]

    print("[!] Building tftf...")
    make(srcdir, args)

    if not os.path.exists(outbin):
        print("[!] Failed to build: %s" % outbin)
        exit(1)

    rename = os.path.join(OUT, "fip-tf-a-tests.bin")
    run(["cp", outbin, rename], cwd=ROOT)

def prepare_vm_image():
    global CROSS_COMPILE
    global VM_IMAGE
    global OUT

    srcdir = VM_IMAGE
    outbin = os.path.join(VM_IMAGE, "build/fvp/debug/tftf.bin")

    args = [
        "CROSS_COMPILE=%s" % CROSS_COMPILE,
        "PLAT=fvp",
        "DEBUG=1",
        "tftf"
    ]

    print("[!] Building vm-image...")
    make(srcdir, args)

    if not os.path.exists(outbin):
        print("[!] Failed to build: %s" % outbin)
        exit(1)

    rename = os.path.join(OUT, "vm-image.bin")
    run(["cp", outbin, rename], cwd=ROOT)

def prepare_tfa():
    global CROSS_COMPILE
    global OUT
    global TRUSTED_FIRMWARE_A

    srcdir = TRUSTED_FIRMWARE_A
    outbin = os.path.join(TRUSTED_FIRMWARE_A, "build/fvp/debug/bl1.bin")

    args = [
        "CROSS_COMPILE=%s" % CROSS_COMPILE,
        "PLAT=fvp",
        "ENABLE_RME=1",
        "FVP_HW_CONFIG_DTS=%s/fdts/fvp-base-gicv3-psci-1t.dts" % TRUSTED_FIRMWARE_A,
        "DEBUG=1",
        "all"
    ]

    print("[!] Building tfa...")
    make(srcdir, args)

    if not os.path.exists(outbin):
        print("[!] Failed to build: %s" % outbin)
        exit(1)

    run(["cp", outbin, OUT], cwd=ROOT)

def prepare_rmm():
    global CROSS_COMPILE
    global OUT
    global RMM

    print("[!] Building rmm...")
    run(["cargo", "build", "--release"], cwd=RMM)
    run(["%sobjcopy" % CROSS_COMPILE, "-O", "binary",
         "%s/aarch64-unknown-none-softfloat/release/fvp" % OUT,
         "%s/aarch64-unknown-none-softfloat/release/rmm.bin" % OUT], cwd=ROOT) 

def prepare_fip():
    global FIPTOOL
    global OUT
    global TRUSTED_FIRMWARE_A
    global TF_A_TESTS

    print("[!] Building fip...")
    make(FIPTOOL)

    fiptool = os.path.join(FIPTOOL, "fiptool")
    tfa = TRUSTED_FIRMWARE_A
    run(["%s" % fiptool,
         "create",
         "--fw-config", "%s/build/fvp/debug/fdts/fvp_fw_config.dtb" % tfa,
         "--tb-fw-config", "%s/build/fvp/debug/fdts/fvp_tb_fw_config.dtb" % tfa,
         "--soc-fw-config", "%s/build/fvp/debug/fdts/fvp_soc_fw_config.dtb" % tfa,
         "--nt-fw-config", "%s/build/fvp/debug/fdts/fvp_nt_fw_config.dtb" % tfa,
         "--hw-config", "%s/build/fvp/debug/fdts/fvp-base-gicv3-psci-1t.dtb" % tfa,
         "--tb-fw", "%s/build/fvp/debug/bl2.bin" % tfa, 
         "--soc-fw", "%s/build/fvp/debug/bl31.bin" % tfa,
         "--rmm-fw", "%s/aarch64-unknown-none-softfloat/release/rmm.bin" % OUT,
         "--nt-fw", "%s/build/fvp/debug/tftf.bin" % TF_A_TESTS,
         "%s/fip-tf-a-tests.bin" % OUT], cwd=ROOT)

def run_fvp():
    global CONFIG
    global FASTMODEL
    global ROOT
    global OUT

    print("[!] Running fvp...")
    run(["./FVP_Base_RevC-2xAEMvA",
         "-C", "bp.flashloader0.fname=%s/fip-tf-a-tests.bin" % OUT,
         "-C", "bp.secureflashloader.fname=%s/bl1.bin" % OUT,
         "--data=%s/vm-image.bin@0x8806c000" % OUT,
         "-f", CONFIG,
         "-Q", "1000"], cwd=FASTMODEL)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="FVP launcher for CCA")
    parser.add_argument("--normal-world", "-nw", help="A normal world component")
    args = parser.parse_args()

    nw_list = ["linux", "tf-a-tests"];
    if not args.normal_world in nw_list:
        print("Please select one of the normal components:")
        print("  " + "\n  ".join(nw_list))
        exit(1)

    if args.normal_world == "tf-a-tests":
        prepare_tftf()
        prepare_tfa()
        prepare_vm_image()
        prepare_rmm()
        prepare_fip()
        run_fvp()
    else:
        assert False, "Not implemented"
