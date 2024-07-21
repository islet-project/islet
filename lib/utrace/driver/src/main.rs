mod toolchain;
mod utils;

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use utrace_common::config;
use utrace_common::UnsafeKind;
use utrace_common::{Record, Records};

struct Analyzer {
    records: Records,
}

impl Analyzer {
    fn new() -> Self {
        Self {
            records: Records::new(),
        }
    }

    fn run(&self) {
        let target_dir = "/home/sangwan/islet/rmm";
        let target_dir = Path::new(&target_dir);
        env::set_current_dir(&target_dir).expect("Failed to change dir to plugin.");

        let out_dir = utrace_common::config::out_dir();
        let path = Path::new(&out_dir);
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
            fs::create_dir_all(&path).unwrap();
        } else {
            fs::create_dir_all(&path).unwrap();
        }

        Command::new("cargo")
            .arg("clean")
            .status()
            .expect("Failed to clean the package.");

        Command::new("rustup")
            .arg("run")
            .arg("utrace")
            .arg("cargo")
            .arg("build")
            .status()
            .expect("Failed to utrace.");
    }

    fn load(&mut self) -> io::Result<()> {
        let out_dir = config::out_dir();
        let out_dir = Path::new(&out_dir);

        for entry in fs::read_dir(&out_dir)? {
            let entry = entry?;
            self.records
                .add(Record::load(entry.path().to_str().unwrap())?);
        }

        Ok(())
    }

    fn report(&mut self) {
        self.load().expect("Failed to read records.");

        println!(
            "{:<20} {:<10} {:<10} {:<10} {:<10}",
            "Crate", "Functions", "Blocks", "Impls", "Traits"
        );
        for record in &self.records {
            let mut items = record.items.clone();
            items.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.name.cmp(&b.name)));

            let mut functions = 0;
            let mut blocks = 0;
            let mut impls = 0;
            let mut traits = 0;

            for item in items {
                match item.kind {
                    UnsafeKind::Function => functions += 1,
                    UnsafeKind::Block => blocks += 1,
                    UnsafeKind::Impl => impls += 1,
                    UnsafeKind::Trait => traits += 1,
                }
            }

            println!(
                "{:<20} {:<10} {:<10} {:<10} {:<10}",
                record.krate, functions, blocks, impls, traits
            );
        }
    }
}

fn main() {
    toolchain::build();
    toolchain::install();

    let mut analyzer = Analyzer::new();
    analyzer.run();
    analyzer.report();
}
