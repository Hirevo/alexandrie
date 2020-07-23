use crate::event_id::EventId;
use crate::file_header::{write_file_header, FILE_MAGIC_EVENT_STREAM};
use crate::raw_event::RawEvent;
use crate::serialization::SerializationSink;
use crate::stringtable::{SerializableString, StringId, StringTableBuilder};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

pub struct ProfilerFiles {
    pub events_file: PathBuf,
    pub string_data_file: PathBuf,
    pub string_index_file: PathBuf,
}

impl ProfilerFiles {
    pub fn new(path_stem: &Path) -> ProfilerFiles {
        ProfilerFiles {
            events_file: path_stem.with_extension("events"),
            string_data_file: path_stem.with_extension("string_data"),
            string_index_file: path_stem.with_extension("string_index"),
        }
    }
}

pub struct Profiler<S: SerializationSink> {
    event_sink: Arc<S>,
    string_table: StringTableBuilder<S>,
    start_time: Instant,
}

impl<S: SerializationSink> Profiler<S> {
    pub fn new(path_stem: &Path) -> Result<Profiler<S>, Box<dyn Error>> {
        let paths = ProfilerFiles::new(path_stem);
        let event_sink = Arc::new(S::from_path(&paths.events_file)?);

        // The first thing in every file we generate must be the file header.
        write_file_header(&*event_sink, FILE_MAGIC_EVENT_STREAM);

        let string_table = StringTableBuilder::new(
            Arc::new(S::from_path(&paths.string_data_file)?),
            Arc::new(S::from_path(&paths.string_index_file)?),
        );

        let profiler = Profiler {
            event_sink,
            string_table,
            start_time: Instant::now(),
        };

        let mut args = String::new();
        for arg in std::env::args() {
            args.push_str(&arg.escape_default().to_string());
            args.push(' ');
        }

        profiler.string_table.alloc_metadata(&*format!(
            r#"{{ "start_time": {}, "process_id": {}, "cmd": "{}" }}"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            std::process::id(),
            args,
        ));

        Ok(profiler)
    }

    #[inline(always)]
    pub fn map_virtual_to_concrete_string(&self, virtual_id: StringId, concrete_id: StringId) {
        self.string_table
            .map_virtual_to_concrete_string(virtual_id, concrete_id);
    }

    #[inline(always)]
    pub fn bulk_map_virtual_to_single_concrete_string<I>(
        &self,
        virtual_ids: I,
        concrete_id: StringId,
    ) where
        I: Iterator<Item = StringId> + ExactSizeIterator,
    {
        self.string_table
            .bulk_map_virtual_to_single_concrete_string(virtual_ids, concrete_id);
    }

    #[inline(always)]
    pub fn alloc_string<STR: SerializableString + ?Sized>(&self, s: &STR) -> StringId {
        self.string_table.alloc(s)
    }

    /// Records an event with the given parameters. The event time is computed
    /// automatically.
    pub fn record_instant_event(&self, event_kind: StringId, event_id: EventId, thread_id: u32) {
        let raw_event =
            RawEvent::new_instant(event_kind, event_id, thread_id, self.nanos_since_start());

        self.record_raw_event(&raw_event);
    }

    /// Creates a "start" event and returns a `TimingGuard` that will create
    /// the corresponding "end" event when it is dropped.
    #[inline]
    pub fn start_recording_interval_event<'a>(
        &'a self,
        event_kind: StringId,
        event_id: EventId,
        thread_id: u32,
    ) -> TimingGuard<'a, S> {
        TimingGuard {
            profiler: self,
            event_id,
            event_kind,
            thread_id,
            start_ns: self.nanos_since_start(),
        }
    }

    fn record_raw_event(&self, raw_event: &RawEvent) {
        self.event_sink
            .write_atomic(std::mem::size_of::<RawEvent>(), |bytes| {
                raw_event.serialize(bytes);
            });
    }

    fn nanos_since_start(&self) -> u64 {
        let duration_since_start = self.start_time.elapsed();
        duration_since_start.as_secs() * 1_000_000_000 + duration_since_start.subsec_nanos() as u64
    }
}

/// When dropped, this `TimingGuard` will record an "end" event in the
/// `Profiler` it was created by.
#[must_use]
pub struct TimingGuard<'a, S: SerializationSink> {
    profiler: &'a Profiler<S>,
    event_id: EventId,
    event_kind: StringId,
    thread_id: u32,
    start_ns: u64,
}

impl<'a, S: SerializationSink> Drop for TimingGuard<'a, S> {
    #[inline]
    fn drop(&mut self) {
        let raw_event = RawEvent::new_interval(
            self.event_kind,
            self.event_id,
            self.thread_id,
            self.start_ns,
            self.profiler.nanos_since_start(),
        );

        self.profiler.record_raw_event(&raw_event);
    }
}

impl<'a, S: SerializationSink> TimingGuard<'a, S> {
    /// This method set a new `event_id` right before actually recording the
    /// event.
    #[inline]
    pub fn finish_with_override_event_id(mut self, event_id: EventId) {
        self.event_id = event_id;
        // Let's be explicit about it: Dropping the guard will record the event.
        drop(self)
    }
}
