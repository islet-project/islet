#!/usr/bin/env python3

import argparse
import errno
import glob
import multiprocessing
import os
import signal
import subprocess
import sys
from abc import ABC, abstractmethod

# Attempt to import from config files.
# Subclasses will need to ensure their specific config_* modules are available
# or handle missing imports appropriately if some paths are not universally defined.
# For now, we assume these will be resolved by the subclass's context.
try:
    from config import *
    # from initrd import prepare_initrd # This will be called by subclasses
except ImportError as e:
    print(f"[!] Warning: Could not import common config in cca_base.py: {e}")

os.makedirs(OUT, exist_ok=True)


class CCAPlatform(ABC):
    def __init__(self):
        parser = self.create_argument_parser() # Create a temp instance to access parser
        self.args = parser.parse_args()

        # Subclasses should initialize their specific config paths in their __init__
        # or ensure they are available globally when their methods run.

    @staticmethod
    def run(cmd, cwd, new_env=None):
        process = subprocess.run(cmd, cwd=cwd,
                           stderr=subprocess.STDOUT,
                           stdout=subprocess.PIPE,
                           universal_newlines=True,
                           env=new_env,
                           check=False)
        if process.returncode != 0:
            print(f"[!] Failed to run: {' '.join(cmd)} @ {cwd}")
            print(process.stdout)
            sys.exit(1)

    @staticmethod
    def make(srcdir, extra=None):
        args = ["make"]
        if extra:
            args += extra
        CCAPlatform.run(args, cwd=srcdir)

    @staticmethod
    def kill(pid):
        try:
            os.kill(pid, signal.SIGTERM)
        except OSError as e:
            if e.errno != errno.ESRCH: # No such process
                print(f"Error sending signal to {pid}: {e}")
                sys.exit(1)

    @staticmethod
    def kill_pid_file(pid_file_path):
        if not os.path.exists(pid_file_path):
            return
        try:
            with open(pid_file_path, "r") as pid_file:
                pid_str = pid_file.read().strip()
                if pid_str.isdigit():
                    CCAPlatform.kill(int(pid_str))
            os.remove(pid_file_path)
        except Exception as e:
            print(f"[!] Error killing PID file {pid_file_path}: {e}")


    @staticmethod
    def custom_signal_handler(signum, frame, pid_file_path=HES_PID):
        print("Signal %s intercepted" % signal.Signals(signum).name)
        CCAPlatform.kill_pid_file(pid_file_path)
        # Call the original handler to ensure default behavior (e.g., exit)
        # For SIGINT and SIGTERM, the default is to exit.
        # If a different default is needed, this might need adjustment.
        if signum == signal.SIGINT:
            signal.default_int_handler(signum, frame)
        elif signum == signal.SIGTERM:
            # Explicitly exit for SIGTERM as default_int_handler is for SIGINT
            sys.exit(1)


    @staticmethod
    def make_single_line(excluded):
        with open(excluded, 'r') as excluded_file:
            excluded_tests = ""
            for i, line in enumerate(excluded_file.readlines()):
                line = line.strip()
                if i == 0:
                    excluded_tests = line
                else:
                    excluded_tests = ",".join([excluded_tests, line])
            return excluded_tests

    @abstractmethod
    def default_optee_build_args(self, target=""):
        pass

    def prepare_tf_a_tests(self, realm):
        srcdir = TF_A_TESTS
        # Assuming TFTF_BIN is a common path or defined by subclass config
        outbin = TFTF_BIN

        args = [
            "CROSS_COMPILE=%s" % CROSS_COMPILE,
            "PLAT=%s" % self.platform_name,  # Use platform_name from subclass
            "DEBUG=1",
            "TESTS=realm-payload",
            "realm",
            "tftf",
        ]

        if realm != "rsi-test":
            args += ["ENABLE_REALM_PAYLOAD_TESTS=1"]
            args += ["BRANCH_PROTECTION=0"]

        print("[!] Building tf-a-tests for %s..." % self.platform_name)
        self.make(srcdir, args)

        if not os.path.exists(outbin):
            print("[!] Failed to build: %s" % outbin)
            sys.exit(1)

        # Pack realm to tftf
        if realm == "rsi-test":
            tftf_max_size = 10485760
            pack_args = [
                "dd",
                "if=%s" % RSI_TEST_BIN,
                "of=%s" % outbin,
                "obs=1",
                "seek=%s" % tftf_max_size,
            ]
            self.run(pack_args, cwd=ROOT)

    def prepare_rsi_test(self):
        self.run(["cargo", "build", "--release"], cwd=RSI_TEST)
        objcopy_cmd = [
            "%sobjcopy" % CROSS_COMPILE,
            "-O",
            "binary",
            "%s/aarch64-unknown-none-softfloat/release/rsi-test" % OUT,
            RSI_TEST_BIN,
        ]
        self.run(objcopy_cmd, cwd=ROOT)

        os.makedirs("%s/realm" % OUT, exist_ok=True)
        self.run(["cp", RSI_TEST_BIN, "%s/realm" % OUT], cwd=ROOT)

        if not os.path.exists(RSI_TEST_BIN):
            print("[!] Failed to build rsi-test")
            sys.exit(1)

    def prepare_realm(self, name):
        print(f"[!] Building realm({name})... ")
        if name == "rsi-test": # Directly use name, self.args.realm might not be set if called differently
            self.prepare_rsi_test()
        else:
            srcdir = os.path.join(REALM, name)
            self.make(srcdir) # 'make' without arguments
            self.make(srcdir, ["install"]) # 'make install'

    def prepare_sdk(self):
        print("[!] Building SDK...")
        # Subclasses might need to pass specific platform args to SDK make
        # TODO: self.make(SDK, [self.platform_name])
        self.make(SDK, ["fvp"])

        print("[!] Building RSI kernel module...")
        self.make(RSI_KO)

    def prepare_islet_hes(self):
        print("[!] Building islet-hes... ")
        self.run(["cargo", "build", "--release"], cwd=HES_APP)

    @abstractmethod
    def prepare_bootloaders(self, rmm, bl33, hes):
        pass

    def get_rmm_features(self):
        args = self.args
        features = []
        if args.rmm == "islet":
            if args.rmm_log_level == "off":
                features += ["--features", "max_level_off"]
            elif args.rmm_log_level == "error":
                features += ["--features", "max_level_error"]
            elif args.rmm_log_level == "warn":
                features += ["--features", "max_level_warn"]
            elif args.rmm_log_level == "info":
                features += ["--features", "max_level_info"]
            elif args.rmm_log_level == "debug":
                features += ["--features", "max_level_debug"]
            else:
                features += ["--features", "max_level_trace"] # default trace

            if args.stat:
                features += ["--features", "stat"]
            if args.normal_world == "acs":
                features += ["--features", "gst_page_table"]

            features += ["--features", self.platform_name]

        if features:
            print(f"[!] Setting {args.rmm} features: {features}")
        return features

    @abstractmethod
    def prepare_tf_rmm(self):
        pass

    def prepare_rmm(self, rmm, features):
        print("[!] Building realm management monitor for FVP: %s" % rmm)
        if rmm == "islet":
            cargo_args = ["cargo", "build", "--release"] + features
            new_env = os.environ.copy()
            new_env["PLATFORM"] = self.platform_name
            self.run(cargo_args, cwd=RMM, new_env=new_env)
            objcopy_cmd = [
                "%sobjcopy" % CROSS_COMPILE,
                "-O",
                "binary",
                "%s/aarch64-unknown-none-softfloat/release/islet-rmm" % OUT,
                "%s/rmm.bin" % OUT,
            ]
            self.run(objcopy_cmd, cwd=ROOT)
        elif rmm == "tf-rmm":
            self.prepare_tf_rmm()
        pass

    @abstractmethod
    def prepare_nw_linux(self):
        pass

    @abstractmethod
    def prepare_tap_network(self):
        pass

    def prepare_kvmtool(self, lkvm="lkvm"):
        print("[!] Building kvmtool...")
        args = [
            "CROSS_COMPILE=%s" % KVMTOOL_CROSS_COMPILE,
            "ARCH=arm64",
            "LIBFDT_DIR=%s/libfdt" % DTC,
            lkvm,
        ]
        self.make(KVMTOOL, args)
        self.run(["cp", "%s/%s" % (KVMTOOL, lkvm), OUT], cwd=ROOT)

    def prepare_kvm_unit_tests(self):
        print("[!] Building kvm-unit-tests...")
        self.run(["./scripts/build-kvm-unit-tests.sh"], cwd=ROOT)
        self.run(["cp", "-R", "arm", "%s/kvm-unit-tests" % OUT], cwd=KVM_UNIT_TESTS)

    def prepare_acs(self, start, end, excluded):
        print("[!] Building ACS...")
        if start == "" and end == "":
            if excluded == "":
                self.run(["./scripts/build-acs.sh"], cwd=ROOT)
            else:
                excluded_tests = self.make_single_line(excluded)
                self.run(["./scripts/build-acs.sh", excluded_tests], cwd=ROOT)
        else:
            if excluded == "":
                self.run(["./scripts/build-acs.sh", start, end], cwd=ROOT)
            else:
                excluded_tests = self.make_single_line(excluded)
                self.run(["./scripts/build-acs.sh", excluded_tests, start, end], cwd=ROOT)

    @abstractmethod
    def prepare_run_arguments(self):
        pass

    @abstractmethod
    def run_tf_a_tests(self):
        pass

    @abstractmethod
    def run_nw_linux(self):
        pass

    @abstractmethod
    def run_nw_linux_net(self):
        pass

    @abstractmethod
    def run_acs(self):
        pass

    def run_islet_hes(self):
        print("[!] Running islet-hes...")
        self.kill_pid_file(HES_PID) # Use HES_PID from base or overridden by subclass
        self.run(["cargo", "run", "--", "-d", "-a", self.hes_address], cwd=HES_APP)

    def place_prebuilt_at_shared(self):
        # Ensure paths like REALM_ROOTFS, PREBUILT are correctly resolved
        # These might be from global config or need to be attributes of self
        self.run([f"cp", f"{REALM_ROOTFS}/rootfs-realm.cpio.gz", SHARED_PATH], cwd=ROOT)
        self.run([f"cp", f"{PREBUILT}/Image", OUT], cwd=ROOT)
        # FVP specific: self.run([f"cp", f"{PREBUILT}/fvp-base-revc.dtb", OUT], cwd=ROOT)
        # QEMU specific: self.run([f"cp", f"{PREBUILT}/qemu-arm64.dtb", OUT], cwd=ROOT) # Example
        # GRUB path might also be platform specific or common
        # self.run([f"cp", PREBUILT_GRUB, OUT], cwd=ROOT) # PREBUILT_GRUB needs to be defined

    def place_script_at_shared(self):
        # Script paths should be well-defined (e.g., relative to ROOT)
        self.run(["cp", LAUNCH_REALM, SHARED_PATH], cwd=ROOT)
        self.run(["cp", LAUNCH_REALM_DEBIAN, SHARED_PATH], cwd=ROOT)
        self.run(["cp", TEST_REALM, SHARED_PATH], cwd=ROOT)
        self.run(["cp", CONFIGURE_NET, SHARED_PATH], cwd=ROOT)
        self.run(["cp", SET_REALM_IP, SHARED_PATH], cwd=ROOT)

    def place_realm_at_shared(self, rmm, realm_ip, platform_tap_ip, platform_ip, no_kvm_unit_tests, no_prebuilt_ml):
        os.makedirs(SHARED_PATH, exist_ok=True)
        self.run([f"cp", f"{REALM_ROOTFS}/rootfs-realm.cpio.gz", SHARED_PATH], cwd=ROOT)
        self.run([f"mv", f"{OUT}/realm/linux.realm", SHARED_PATH], cwd=ROOT)
        self.run([f"mv", f"{OUT}/lkvm", SHARED_PATH], cwd=ROOT)
        self.place_script_at_shared()
        if not no_kvm_unit_tests:
            self.run([f"cp", "-R", f"{OUT}/kvm-unit-tests", SHARED_PATH], cwd=ROOT)

        if realm_ip and platform_ip and platform_tap_ip:
            # Use platform_ip for FVP_IP/QEMU_IP replacement
            self.run(["sed", "-i", "-e", f"s/FVP_IP/{platform_ip}/g", f"{SHARED_PATH}/configure-net.sh"], cwd=ROOT)
            self.run(["sed", "-i", "-e", f"s/FVP_TAP_IP/{platform_tap_ip}/g", f"{SHARED_PATH}/configure-net.sh"], cwd=ROOT)
            self.run(["sed", "-i", "-e", f"s/FVP_TAP_IP/{platform_tap_ip}/g", f"{SHARED_PATH}/set-realm-ip.sh"], cwd=ROOT)
            self.run(["sed", "-i", "-e", f"s/REALM_IP/{realm_ip}/g", f"{SHARED_PATH}/set-realm-ip.sh"], cwd=ROOT)
            self.run(["sed", "-i", "-e", f"s/FVP_TAP_IP/{platform_tap_ip}/g", f"{SHARED_PATH}/launch-realm-debian.sh"], cwd=ROOT)

        if not no_prebuilt_ml:
            os.makedirs(SHARED_EXAMPLES_PATH, exist_ok=True)
            self.run(
                [f"cp", "-R", f"{EXAMPLES}/confidential-ml", SHARED_EXAMPLES_PATH],
                cwd=ROOT,
            )
            self.run(
                [
                    f"cp",
                    f"{PREBUILT_EXAMPLES}/confidential-ml/device/device.exe",
                    f"{SHARED_EXAMPLES_PATH}/confidential-ml/device/device.exe",
                ],
                cwd=ROOT,
            )
            self.run(
                [f"cp", "-R", f"{PREBUILT_EXAMPLES}/lib", SHARED_EXAMPLES_PATH],
                cwd=ROOT,
            )
            self.run(
                [
                    "tar",
                    "-zxvf",
                    f"{SHARED_EXAMPLES_PATH}/lib/libtensorflowlite.tar.gz",
                    "-C",
                    f"{SHARED_EXAMPLES_PATH}/lib/",
                ],
                cwd=ROOT,
            )

    @abstractmethod
    def _clean_platform_tf_a(self):
        """Platform-specific cleaning for TF-A."""
        pass

    @abstractmethod
    def _clean_nw_linux(self):
        """Platform-specific cleaning for Normal World Linux."""
        pass

    @abstractmethod
    def _clean_tf_rmm(self):
        """Platform-specific cleaning for TF-RMM."""
        pass

    def clean_repo(self, target):
        clean_list = ["all", "tf-a", "tf-a-tests", "realm-linux", "kvmtool", "nw-linux", "acs", "tf-rmm", "islet"]
        if target not in clean_list:
            print("Please select one of the clean list:")
            print("  " + "\n  ".join(clean_list))
            sys.exit(1)

        if target == "all" or target == "tf-a":
            self._clean_platform_tf_a()
        if target == "all" or target == "tf-a-tests":
            self.run(["make", "distclean"], cwd=TF_A_TESTS)
        if target == "all" or target == "realm-linux":
            self.run(["make", "clean"], cwd=REALM_LINUX)
        if target == "all" or target == "kvmtool":
            self.run(["make", "clean"], cwd=KVMTOOL)
        if target == "all" or target == "nw-linux":
            self._clean_nw_linux()
        if target == "all" or target == "acs":
            self.run(["rm", "-rf", "build"], cwd=ACS)
        if target == "all" or target == "tf-rmm":
            self._clean_tf_rmm()
        if target == "all" or target == "islet":
            # islet-rmm build output is in OUT/aarch64-unknown-none-softfloat
            # OUT should be used here as ROOT might be different if script is not in ROOT
            self.run(["rm", "-rf", "out/aarch64-unknown-none-softfloat"], cwd=ROOT) # Or ROOT if OUT is relative to it

    def get_all_realms(self):
        realms = []
        for dirp in glob.glob(os.path.join(REALM, "*/")):
            realms.append(os.path.basename(dirp.rstrip("/")))
        return sorted(realms)

    def validate_args(self):
        args = self.args
        nw_list = ["linux", "linux-net", "tf-a-tests", "acs"]
        if not args.use_prebuilt and args.normal_world not in nw_list:
            print("Please select one of the normal components:")
            print("  " + "\n  ".join(nw_list))
            sys.exit(1)

        rmm_list = ["islet", "trp", "tf-rmm"]
        if args.rmm not in rmm_list:
            print("Please select one of the rmm components:")
            print("  " + "\n  ".join(rmm_list))
            sys.exit(1)

        if args.realm is not None:
            realms = self.get_all_realms()
            if args.realm not in realms:
                print("Please select one of the realms:")
                print("  " + "\n  ".join(realms))
                sys.exit(1)

    @abstractmethod
    def add_platform_arguments(self, parser):
        """Add platform-specific command-line arguments to the parser."""
        pass

    def create_argument_parser(self):
        parser = argparse.ArgumentParser(description=f"{self.platform_name.upper()} launcher for CCA")
        parser.add_argument("--normal-world", "-nw", help="A normal world component")
        parser.add_argument("--debug", "-d", help="Using debug component", action="store_true")
        parser.add_argument("--run-only", "-ro", help=f"Running {self.platform_name} without building", action="store_true")
        parser.add_argument("--build-only", "-bo", help="Building components without running", action="store_true")
        parser.add_argument("--clean", "-c", help="Clean the repo (pass `target name` or `all`)", default="")
        parser.add_argument("--realm-launch", help="Execute realm launch on boot", action="store_true")
        parser.add_argument("--realm", "-rm", help="A sample realm")
        parser.add_argument("--rmm", "-rmm", help="A realm management monitor (islet, trp, tf-rmm)", default="islet")
        parser.add_argument("--use-prebuilt", help="Use prebuilt binary (realm-linux, nw-linux, lkvm, etc)", action="store_true")
        parser.add_argument("--no-prebuilt-ml", help="Not using the prebuilt confidential-ml example", action="store_true")
        parser.add_argument("--no-kvm-unit-tests", help="Do not build kvm unit tests", action="store_true")
        parser.add_argument("--no-sdk", help="Do not build sdk", action="store_true")
        parser.add_argument("--hes", help="Run with hes", action="store_true")

        # Network arguments - common names, but platform_ip refers to fvp-ip or qemu-ip
        parser.add_argument("--host-ip", "-hip", help="the ip address of host machine", default="192.168.10.15")
        parser.add_argument("--host-tap-ip", "-htip", help="the ip address of the tap device in host", default="192.168.10.1")
        parser.add_argument("--platform-ip", "-pip", help="the ip address that is going to be assigned to the platform host (e.g. fvp, qemu)", default="192.168.10.5")  # Renamed conceptually
        parser.add_argument("--platform-tap-ip", "-ptip", help="the ip address for tap device in platform", default="192.168.20.1")  # Renamed conceptually
        parser.add_argument("--realm-ip", "-reip", help="the ip address for realm", default="192.168.20.10")
        parser.add_argument("--route-ip", "-roip", help="the route ip for platform", default="192.168.20.0")
        parser.add_argument("--gateway", "-gw", help="the gateway ip for host machine", default="192.168.10.1")
        parser.add_argument("--ifname", "-if", help="the main interface name of host machine", default="eth0")
        parser.add_argument("--rmm-log-level", help="Determine RMM's log-level. Choose among (off, error, warn, info, debug, trace)", default="trace")
        parser.add_argument("--stat", help="Enable stat to check memory used size per command", action="store_true")
        parser.add_argument("--selected-tests", "-st", help="Select the first and end test name separated by ';'", default="")
        parser.add_argument("--excluded-tests", "-et", help="File name which contains the list of ACS tests to be excluded", default="")

        self.add_platform_arguments(parser) # Allow subclass to add more
        return parser

    def execute(self):
        args = self.args
        if args.clean != "":
            self.clean_repo(args.clean)
            sys.exit(0)

        self.validate_args()

        if not args.run_only:
            features = self.get_rmm_features()

            if args.hes:
                self.prepare_islet_hes()

            self.prepare_rmm(args.rmm, features)

            if args.realm is not None:
                self.prepare_realm(args.realm)

            if args.use_prebuilt:
                # PREBUILT_EDK2 needs to be correctly defined for the platform
                self.prepare_bootloaders(args.rmm, self.PREBUILT_EDK2, args.hes)
                self.place_script_at_shared()
                self.place_prebuilt_at_shared() # This might need platform-specific adjustments (e.g. DTB)
            elif args.normal_world == "tf-a-tests":
                self.prepare_tf_a_tests(args.realm)
                self.prepare_bootloaders(args.rmm, TFTF_BIN, args.hes)
            elif args.normal_world == "acs":
                if args.selected_tests == "":
                    self.prepare_acs("", "", args.excluded_tests)
                    self.prepare_bootloaders(args.rmm, ACS_HOST, args.hes) # ACS_HOST needs definition
                else:
                    selected_tests = args.selected_tests
                    tests = selected_tests.split(";")
                    test_num = len(tests)
                    if test_num == 1:
                        self.prepare_acs(tests[0], tests[0], args.excluded_tests)
                    elif test_num == 2:
                        self.prepare_acs(tests[0], tests[1], args.excluded_tests)
                    else:
                        print("[!] Pass one or two test names separated by ';'")
                        sys.exit(1)
                    self.prepare_bootloaders(args.rmm, ACS_HOST, args.hes)
            else: # linux or linux-net
                self.prepare_kvmtool()
                if not args.no_kvm_unit_tests:
                    self.prepare_kvm_unit_tests()
                # FVP specific: self.prepare_grub_config(args.rmm) # This was FVP specific
                self.prepare_nw_linux()
                self.prepare_bootloaders(args.rmm, self.PREBUILT_EDK2, args.hes)

                if args.realm is not None:
                    self.place_realm_at_shared(args.rmm, args.realm_ip, args.platform_tap_ip, args.platform_ip, args.no_kvm_unit_tests, args.no_prebuilt_ml)
                    if not args.no_sdk:
                        self.prepare_sdk()

        if not args.build_only and args.hes:
            # Pass HES_PID if it can be platform-specific, otherwise use default
            signal.signal(signal.SIGTERM, lambda signum, frame: self.custom_signal_handler(signum, frame, HES_PID))
            signal.signal(signal.SIGINT, lambda signum, frame: self.custom_signal_handler(signum, frame, HES_PID))
            self.run_islet_hes()

        if not args.build_only:
            if args.normal_world == "tf-a-tests":
                self.run_tf_a_tests()
            elif args.normal_world == "linux":
                self.run_nw_linux()
            elif args.normal_world == "linux-net":
                self.run_nw_linux_net()
            elif args.normal_world == "acs":
                self.run_acs()

    @property
    @abstractmethod
    def platform_name(self):
        """Return the platform name (e.g., 'fvp', 'qemu')."""
        pass

    @property
    @abstractmethod
    def hes_address(self):
        """The TCP address the HES implementation should connect to."""
        pass

    @staticmethod
    def main(platform_class):
        """Static main method to be called by subclass scripts."""
        # Platform specific config (e.g., config_fvp, config_qemu) must be loaded
        # before creating the parser or the platform instance, as they define
        # paths used by the base class and argument defaults.
        # This is a bit tricky as the base class shouldn't know about fvp/qemu specifics.
        # A common pattern is for the subclass script to import its config first.
        platform_instance = platform_class()
        platform_instance.execute()
