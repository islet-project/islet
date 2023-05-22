const INCLUDE_PATHS: [&str; 2] = [
    "lib/qcbor/inc",
    "lib/token_verifier",
];

const SOURCE_FILES: [&str; 7] = [
    "lib/qcbor/src/ieee754.c",
    "lib/qcbor/src/qcbor_decode.c",
    "lib/qcbor/src/qcbor_encode.c",
    "lib/qcbor/src/qcbor_err_to_str.c",
    "lib/qcbor/src/UsefulBuf.c",
    "lib/token_verifier/token_dumper.c",
    "lib/token_verifier/token_verifier.c",
];

const HEADER_FILES: [&str; 3] = [
    "lib/token_verifier/attest_defines.h",
    "lib/token_verifier/token_dumper.h",
    "lib/token_verifier/token_verifier.h",
];

// this is required for stupid clang that can't find gcc headers
const BINGDEN_ARGS: [&str; 3] = [
    "-I/usr/include/x86_64-linux-gnu",
    "-D__x86_64__",
    "-D__LP64__",
];

fn main()
{
    // get rid of cross_compile leftover from kernel,
    // cross compiler is chosen from cargo config
    std::env::remove_var("CROSS_COMPILE");

    // generic
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);

    // libtoken
    cc::Build::new()
        .files(&SOURCE_FILES)
        .includes(&INCLUDE_PATHS)
        .compile("libtoken.a");

    for file in SOURCE_FILES {
        println!("cargo:rerun-if-changed={}", file);
    }

    println!("cargo:rustc-link-bin=static=token");

    // bindgen, C token API
    let mut builder = bindgen::builder().clang_args(&BINGDEN_ARGS);

    for header in HEADER_FILES {
        builder = builder.header(header);
    }

    for include in INCLUDE_PATHS {
        let arg = format!("-I{}", include);
        builder = builder.clang_arg(arg);
    }

    let bindings = builder.generate().unwrap();
    bindings.write_to_file("src/token_c/bindgen.rs").unwrap();
}
