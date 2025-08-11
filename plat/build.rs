use std::env;

fn main() {
    let platform = env::var("PLATFORM").unwrap_or_else(|_| {
        "fvp".to_string()
        //panic!("Please set the PLATFORM environment variable (e.g., PLATFORM=fvp)")
    });

    let memory_file = format!("plat/{}/memory.x", platform);
    println!("cargo:rustc-link-arg=-T{}", memory_file);

    println!("cargo:rustc-cfg=plat_{}", platform);

    // Re-run build if platform or memory.x changes
    println!("cargo:rerun-if-env-changed=PLATFORM");
    println!("cargo:rerun-if-changed={}", memory_file);
}
