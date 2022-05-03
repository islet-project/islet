use crate::io::Write;
use log::{Level, LevelFilter, Metadata, Record};

struct SimpleLogger;
extern crate alloc;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            if record.metadata().level() <= Level::Warn {
                crate::eprintln!(
                    "[{}]{} -- {}",
                    record.level(),
                    record.target(),
                    record.args()
                );
            } else {
                crate::println!(
                    "[{}]{} -- {}",
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn register_global_logger(maxlevel: LevelFilter) {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(maxlevel);
}
