//! Print logs as ndjson.

use js_sys::Object;
use log::{kv, Level, LevelFilter, Log, Metadata, Record};
use wasm_bindgen::prelude::*;

use std::collections::HashMap;

/// Start logging.
pub(crate) fn start<F>(level: LevelFilter) {
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
            let args = format!("{}", record.args()).into();
            let line = format_line(&record).into();

            match format_kv_pairs(&record) {
                Some(obj) => match record.level() {
                    Level::Error => web_sys::console::error_3(&args, &obj, &line),
                    Level::Warn => web_sys::console::warn_3(&args, &obj, &line),
                    Level::Info => web_sys::console::info_3(&args, &obj, &line),
                    _ => web_sys::console::debug_3(&args, &obj, &line),
                },
                None => match record.level() {
                    Level::Error => web_sys::console::error_2(&args, &line),
                    Level::Warn => web_sys::console::warn_2(&args, &line),
                    Level::Info => web_sys::console::info_2(&args, &line),
                    _ => web_sys::console::debug_2(&args, &line),
                },
            }
        }
    }
    fn flush(&self) {}
}

fn format_line(record: &Record<'_>) -> String {
    match (record.file(), record.line()) {
        (Some(file), Some(line)) => format!("({}:{})", file, line),
        _ => String::new(),
    }
}

fn format_kv_pairs(record: &Record) -> Option<Object> {
    struct Visitor {
        hashmap: Option<HashMap<String, String>>,
    }

    impl<'kvs> kv::Visitor<'kvs> for Visitor {
        fn visit_pair(
            &mut self,
            key: kv::Key<'kvs>,
            val: kv::Value<'kvs>,
        ) -> Result<(), kv::Error> {
            if self.hashmap.is_none() {
                self.hashmap = Some(HashMap::new())
            }
            let hm = self.hashmap.as_mut().unwrap();
            hm.insert(key.to_string(), val.to_string());
            Ok(())
        }
    }

    impl Visitor {
        fn new() -> Self {
            Self { hashmap: None }
        }
    }

    let mut visitor = Visitor::new();
    record.key_values().visit(&mut visitor).unwrap();

    match visitor.hashmap.as_ref() {
        Some(hashmap) => {
            let val = JsValue::from_serde(hashmap).unwrap();
            Some(Object::from(val))
        }
        None => None,
    }
}
