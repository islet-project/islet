use crate::utils::copy_dir;

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use utrace_common::config;

pub fn build() {
    let plugin_dir = config::plugin_dir();
    let plugin_dir = Path::new(&plugin_dir);
    env::set_current_dir(&plugin_dir).expect("Failed to change dir to plugin.");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build plugin.");
}

pub fn install() {
    // bin
    let toolchain_dir = config::toolchain_dir();
    let bin_dir = Path::new(&toolchain_dir).join("bin");
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir).expect("Failed to create toolchain/bin dir.");
    }

    let plugin_bin = config::plugin_bin();
    let plugin_bin = Path::new(&plugin_bin);
    let target_bin = Path::new(&bin_dir).join("rustc");

    fs::copy(&plugin_bin, &target_bin).expect("Failed to copy a plugin.");

    // lib
    let sysroot = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("Failed to get sysroot")
        .stdout;
    let sysroot_lib = Path::new(String::from_utf8_lossy(&sysroot).trim()).join("lib");

    let lib_dir = Path::new(&toolchain_dir).join("lib");
    if !lib_dir.exists() {
        fs::create_dir_all(&bin_dir).expect("Failed to create toolchain/lib dir.");
    }

    copy_dir(&sysroot_lib, lib_dir).unwrap();

    // link
    Command::new("rustup")
        .arg("toolchain")
        .arg("link")
        .arg("utrace")
        .arg(&toolchain_dir)
        .status()
        .expect("Failed to link toolchain");
}
