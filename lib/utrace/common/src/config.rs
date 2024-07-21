pub fn root_dir() -> String {
    std::env::var("UTRACE_DIR").unwrap()
}

pub fn plugin_dir() -> String {
    format!("{}/plugin", root_dir())
}

pub fn plugin_bin() -> String {
    format!(
        "{}/target/x86_64-unknown-linux-gnu/release/utrace_plugin",
        root_dir()
    )
}

pub fn toolchain_dir() -> String {
    format!("{}/toolchain", root_dir())
}

pub fn out_dir() -> String {
    format!("{}/out", root_dir())
}
