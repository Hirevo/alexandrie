//! Pretty print logs.

use log::{kv, Level, LevelFilter, Log, Metadata, Record};
use std::io::{self, StdoutLock, Write};

// ANSI term codes.
const RESET: &'static str = "\x1b[0m";
const BOLD: &'static str = "\x1b[1m";
const RED: &'static str = "\x1b[31m";
const GREEN: &'static str = "\x1b[32m";
const YELLOW: &'static str = "\x1b[33m";

/// Start logging.
pub(crate) fn start(level: LevelFilter) {
    let logger = Box::new(Logger {});
    log::set_boxed_logger(logger).expect("Could not start logging");
    log::set_max_level(level);
}

#[derive(Debug)]
pub(crate) struct Logger {}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            format_src(&mut handle, &record);
            write!(handle, " {}", &record.args()).unwrap();
            format_kv_pairs(&mut handle, &record);
            writeln!(&mut handle, "").unwrap();
        }
    }
    fn flush(&self) {}
}

fn format_kv_pairs<'b>(mut out: &mut StdoutLock<'b>, record: &Record) {
    struct Visitor<'a, 'b> {
        stdout: &'a mut StdoutLock<'b>,
    }

    impl<'kvs, 'a, 'b> kv::Visitor<'kvs> for Visitor<'a, 'b> {
        fn visit_pair(
            &mut self,
            key: kv::Key<'kvs>,
            val: kv::Value<'kvs>,
        ) -> Result<(), kv::Error> {
            write!(self.stdout, "\n    {}{}{} {}", BOLD, key, RESET, val).unwrap();
            Ok(())
        }
    }

    let mut visitor = Visitor { stdout: &mut out };
    record.key_values().visit(&mut visitor).unwrap();
}

fn format_src(out: &mut StdoutLock<'_>, record: &Record<'_>) {
    let msg = record.target();
    match record.level() {
        Level::Trace | Level::Debug | Level::Info => {
            write!(out, "{}{}{}{}", GREEN, BOLD, msg, RESET).unwrap();
        }
        Level::Warn => write!(out, "{}{}{}{}", YELLOW, BOLD, msg, RESET).unwrap(),
        Level::Error => write!(out, "{}{}{}{}", RED, BOLD, msg, RESET).unwrap(),
    }
}
