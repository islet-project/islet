#!/usr/bin/env python3

import argparse
import os
import tarfile
import shutil

def prepare_initrd(root, out_path, auto_launch=False, net_config=False, fvp_ip="192.168.10.5", host_ip="192.168.10.1"):
    """
    Process rootfs and create cpio archive.

    :param root: Path to the root directory
    :param out_path: Path to the output directory
    :param auto_launch: Enable auto_launch (default: False)
    :param net_config: Enable net_config (default: False)
    :param fvp_ip: FVP IP address (default: 192.168.10.5)
    :param host_ip: HOST IP address (default: 192.168.10.1)
    """
    # Create or clean the rootfs directory
    rootfs_dir = os.path.join(out_path, "rootfs")
    if os.path.exists(rootfs_dir):
    #    print("Cleaning existing rootfs directory...")
        shutil.rmtree(rootfs_dir)  # Remove all contents inside rootfs
    #else:
    #    print("Creating rootfs directory...")
    os.makedirs(rootfs_dir)  # Create rootfs directory if it doesn't exist

    # Unpack the tarball
    tarball_path = os.path.join(root, "assets", "rootfs", "rootfs-linux.tar.bz2")
    if os.path.exists(tarball_path):
        with tarfile.open(tarball_path, "r:bz2") as tar:
            tar.extractall(path=rootfs_dir)
    #    print(f"Unpacked {tarball_path} to {rootfs_dir}")
    else:
        print(f"Error: {tarball_path} does not exist.")
        exit(1)

    # Execute auto_launch if provided
    if auto_launch:
        launch_realm_script = os.path.join(root, "scripts", "fvp", "launch-realm.sh")
        if os.path.exists(launch_realm_script):
            shutil.copy(launch_realm_script, os.path.join(rootfs_dir, "etc", "init.d", "S50launch"))
            #print(f"Copied {launch_realm_script} to {rootfs_dir}/etc/init.d/S50launch")
        else:
            print(f"Error: {launch_realm_script} does not exist.")
            exit(1)

    # Execute net_config if provided
    if net_config:
        interfaces_path = os.path.join(rootfs_dir, "etc", "network", "interfaces")
        with open(interfaces_path, "a") as f:
            f.write("\nauto eth0\niface eth0 inet static\naddress {}\nnetmask 255.255.255.0\ngateway {}\n".format(fvp_ip, host_ip))
        dhcp_script_path = os.path.join(rootfs_dir, "etc", "network", "if-up.d", "dhcp")
        if os.path.exists(dhcp_script_path):
            os.remove(dhcp_script_path)
        #print(f"Updated network configuration in {interfaces_path}")

    # Create cpio archive
    cpio_path = os.path.join(out_path, "rootfs.cpio.gz")
    with open(cpio_path, "wb") as f:
        os.chdir(rootfs_dir)
        os.system("find . | cpio -H newc -o | gzip -c > {}".format(cpio_path))
    #print(f"Created cpio archive at {cpio_path}")

# Example usage
if __name__ == "__main__":
    # Argument parsing
    parser = argparse.ArgumentParser(description="Process rootfs and create cpio archive.")
    parser.add_argument("ROOT", help="Path to the root directory")
    parser.add_argument("OUT_PATH", help="Path to the output directory")
    parser.add_argument("--auto_launch", nargs='?', const="auto_launch", default="", help="Enable auto_launch (optional)")
    parser.add_argument("--net_config", nargs='?', const="net_config", default="", help="Enable net_config (optional)")
    parser.add_argument("--FVP_IP", nargs='?', default="192.168.10.5", help="FVP IP address (default: 192.168.10.5)")
    parser.add_argument("--HOST_IP", nargs='?', default="192.168.10.15", help="HOST IP address (default: 192.168.10.15)")
    args = parser.parse_args()

    prepare_initrd(
        args.ROOT,
        args.OUT_PATH,
        arg.auto_launch,
        arg.net_config,
        arg.FVP_IP,
        arg.HOST_IPO
    )
