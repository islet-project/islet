extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config: cbindgen::Config = Default::default();
    config.header = Some(
        "// Copyright (c) 2023 Samsung Electronics Co., Ltd. All Rights Reserved.".to_string(),
    );
    config.pragma_once = true;
    config.documentation = true;
    config.documentation_style = cbindgen::DocumentationStyle::Cxx;

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings.")
        .write_to_file("include/islet.h");
}
