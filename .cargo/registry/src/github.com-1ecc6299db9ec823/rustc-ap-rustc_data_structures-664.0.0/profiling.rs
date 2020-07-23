//! # Rust Compiler Self-Profiling
//!
//! This module implements the basic framework for the compiler's self-
//! profiling support. It provides the `SelfProfiler` type which enables
//! recording "events". An event is something that starts and ends at a given
//! point in time and has an ID and a kind attached to it. This allows for
//! tracing the compiler's activity.
//!
//! Internally this module uses the custom tailored [measureme][mm] crate for
//! efficiently recording events to disk in a compact format that can be
//! post-processed and analyzed by the suite of tools in the `measureme`
//! project. The highest priority for the tracing framework is on incurring as
//! little overhead as possible.
//!
//!
//! ## Event Overview
//!
//! Events have a few properties:
//!
//! - The `event_kind` designates the broad category of an event (e.g. does it
//!   correspond to the execution of a query provider or to loading something
//!   from the incr. comp. on-disk cache, etc).
//! - The `event_id` designates the query invocation or function call it
//!   corresponds to, possibly including the query key or function arguments.
//! - Each event stores the ID of the thread it was recorded on.
//! - The timestamp stores beginning and end of the event, or the single point
//!   in time it occurred at for "instant" events.
//!
//!
//! ## Event Filtering
//!
//! Event generation can be filtered by event kind. Recording all possible
//! events generates a lot of data, much of which is not needed for most kinds
//! of analysis. So, in order to keep overhead as low as possible for a given
//! use case, the `SelfProfiler` will only record the kinds of events that
//! pass the filter specified as a command line argument to the compiler.
//!
//!
//! ## `event_id` Assignment
//!
//! As far as `measureme` is concerned, `event_id`s are just strings. However,
//! it would incur too much overhead to generate and persist each `event_id`
//! string at the point where the event is recorded. In order to make this more
//! efficient `measureme` has two features:
//!
//! - Strings can share their content, so that re-occurring parts don't have to
//!   be copied over and over again. One allocates a string in `measureme` and
//!   gets back a `StringId`. This `StringId` is then used to refer to that
//!   string. `measureme` strings are actually DAGs of string components so that
//!   arbitrary sharing of substrings can be done efficiently. This is useful
//!   because `event_id`s contain lots of redundant text like query names or
//!   def-path components.
//!
//! - `StringId`s can be "virtual" which means that the client picks a numeric
//!   ID according to some application-specific scheme and can later make that
//!   ID be mapped to an actual string. This is used to cheaply generate
//!   `event_id`s while the events actually occur, causing little timing
//!   distortion, and then later map those `StringId`s, in bulk, to actual
//!   `event_id` strings. This way the largest part of the tracing overhead is
//!   localized to one contiguous chunk of time.
//!
//! How are these `event_id`s generated in the compiler? For things that occur
//! infrequently (e.g. "generic activities"), we just allocate the string the
//! first time it is used and then keep the `StringId` in a hash table. This
//! is implemented in `SelfProfiler::get_or_alloc_cached_string()`.
//!
//! For queries it gets more interesting: First we need a unique numeric ID for
//! each query invocation (the `QueryInvocationId`). This ID is used as the
//! virtual `StringId` we use as `event_id` for a given event. This ID has to
//! be available both when the query is executed and later, together with the
//! query key, when we allocate the actual `event_id` strings in bulk.
//!
//! We could make the compiler generate and keep track of such an ID for each
//! query invocation but luckily we already have something that fits all the
//! the requirements: the query's `DepNodeIndex`. So we use the numeric value
//! of the `DepNodeIndex` as `event_id` when recording the event and then,
//! just before the query context is dropped, we walk the entire query cache
//! (which stores the `DepNodeIndex` along with the query key for each
//! invocation) and allocate the corresponding strings together with a mapping
//! for `DepNodeIndex as StringId`.
//!
//! [mm]: https://github.com/rust-lang/measureme/

use crate::cold_path;
use crate::fx::FxHashMap;

use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::convert::Into;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::time::{Duration, Instant};

use measureme::{EventId, EventIdBuilder, SerializableString, StringId};
use parking_lot::RwLock;

cfg_if! {
    if #[cfg(any(windows, target_os = "wasi"))] {
        /// FileSerializationSink is faster on Windows
        type SerializationSink = measureme::FileSerializationSink;
    } else if #[cfg(target_arch = "wasm32")] {
        type SerializationSink = measureme::ByteVecSink;
    } else {
        /// MmapSerializatioSink is faster on macOS and Linux
        type SerializationSink = measureme::MmapSerializationSink;
    }
}

type Profiler = measureme::Profiler<SerializationSink>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ProfileCategory {
    Parsing,
    Expansion,
    TypeChecking,
    BorrowChecking,
    Codegen,
    Linking,
    Other,
}

bitflags::bitflags! {
    struct EventFilter: u32 {
        const GENERIC_ACTIVITIES = 1 << 0;
        const QUERY_PROVIDERS    = 1 << 1;
        const QUERY_CACHE_HITS   = 1 << 2;
        const QUERY_BLOCKED      = 1 << 3;
        const INCR_CACHE_LOADS   = 1 << 4;

        const QUERY_KEYS         = 1 << 5;
        const FUNCTION_ARGS      = 1 << 6;
        const LLVM               = 1 << 7;

        const DEFAULT = Self::GENERIC_ACTIVITIES.bits |
                        Self::QUERY_PROVIDERS.bits |
                        Self::QUERY_BLOCKED.bits |
                        Self::INCR_CACHE_LOADS.bits;

        const ARGS = Self::QUERY_KEYS.bits | Self::FUNCTION_ARGS.bits;
    }
}

// keep this in sync with the `-Z self-profile-events` help message in librustc_session/options.rs
const EVENT_FILTERS_BY_NAME: &[(&str, EventFilter)] = &[
    ("none", EventFilter::empty()),
    ("all", EventFilter::all()),
    ("default", EventFilter::DEFAULT),
    ("generic-activity", EventFilter::GENERIC_ACTIVITIES),
    ("query-provider", EventFilter::QUERY_PROVIDERS),
    ("query-cache-hit", EventFilter::QUERY_CACHE_HITS),
    ("query-blocked", EventFilter::QUERY_BLOCKED),
    ("incr-cache-load", EventFilter::INCR_CACHE_LOADS),
    ("query-keys", EventFilter::QUERY_KEYS),
    ("function-args", EventFilter::FUNCTION_ARGS),
    ("args", EventFilter::ARGS),
    ("llvm", EventFilter::LLVM),
];

/// Something that uniquely identifies a query invocation.
pub struct QueryInvocationId(pub u32);

/// A reference to the SelfProfiler. It can be cloned and sent across thread
/// boundaries at will.
#[derive(Clone)]
pub struct SelfProfilerRef {
    // This field is `None` if self-profiling is disabled for the current
    // compilation session.
    profiler: Option<Arc<SelfProfiler>>,

    // We store the filter mask directly in the reference because that doesn't
    // cost anything and allows for filtering with checking if the profiler is
    // actually enabled.
    event_filter_mask: EventFilter,

    // Print verbose generic activities to stdout
    print_verbose_generic_activities: bool,

    // Print extra verbose generic activities to stdout
    print_extra_verbose_generic_activities: bool,
}

impl SelfProfilerRef {
    pub fn new(
        profiler: Option<Arc<SelfProfiler>>,
        print_verbose_generic_activities: bool,
        print_extra_verbose_generic_activities: bool,
    ) -> SelfProfilerRef {
        // If there is no SelfProfiler then the filter mask is set to NONE,
        // ensuring that nothing ever tries to actually access it.
        let event_filter_mask =
            profiler.as_ref().map(|p| p.event_filter_mask).unwrap_or(EventFilter::empty());

        SelfProfilerRef {
            profiler,
            event_filter_mask,
            print_verbose_generic_activities,
            print_extra_verbose_generic_activities,
        }
    }

    // This shim makes sure that calls only get executed if the filter mask
    // lets them pass. It also contains some trickery to make sure that
    // code is optimized for non-profiling compilation sessions, i.e. anything
    // past the filter check is never inlined so it doesn't clutter the fast
    // path.
    #[inline(always)]
    fn exec<F>(&self, event_filter: EventFilter, f: F) -> TimingGuard<'_>
    where
        F: for<'a> FnOnce(&'a SelfProfiler) -> TimingGuard<'a>,
    {
        #[inline(never)]
        fn cold_call<F>(profiler_ref: &SelfProfilerRef, f: F) -> TimingGuard<'_>
        where
            F: for<'a> FnOnce(&'a SelfProfiler) -> TimingGuard<'a>,
        {
            let profiler = profiler_ref.profiler.as_ref().unwrap();
            f(&**profiler)
        }

        if unlikely!(self.event_filter_mask.contains(event_filter)) {
            cold_call(self, f)
        } else {
            TimingGuard::none()
        }
    }

    /// Start profiling a verbose generic activity. Profiling continues until the
    /// VerboseTimingGuard returned from this call is dropped. In addition to recording
    /// a measureme event, "verbose" generic activities also print a timing entry to
    /// stdout if the compiler is invoked with -Ztime or -Ztime-passes.
    pub fn verbose_generic_activity<'a>(
        &'a self,
        event_label: &'static str,
    ) -> VerboseTimingGuard<'a> {
        let message =
            if self.print_verbose_generic_activities { Some(event_label.to_owned()) } else { None };

        VerboseTimingGuard::start(message, self.generic_activity(event_label))
    }

    /// Start profiling a extra verbose generic activity. Profiling continues until the
    /// VerboseTimingGuard returned from this call is dropped. In addition to recording
    /// a measureme event, "extra verbose" generic activities also print a timing entry to
    /// stdout if the compiler is invoked with -Ztime-passes.
    pub fn extra_verbose_generic_activity<'a, A>(
        &'a self,
        event_label: &'static str,
        event_arg: A,
    ) -> VerboseTimingGuard<'a>
    where
        A: Borrow<str> + Into<String>,
    {
        let message = if self.print_extra_verbose_generic_activities {
            Some(format!("{}({})", event_label, event_arg.borrow()))
        } else {
            None
        };

        VerboseTimingGuard::start(message, self.generic_activity_with_arg(event_label, event_arg))
    }

    /// Start profiling a generic activity. Profiling continues until the
    /// TimingGuard returned from this call is dropped.
    #[inline(always)]
    pub fn generic_activity(&self, event_label: &'static str) -> TimingGuard<'_> {
        self.exec(EventFilter::GENERIC_ACTIVITIES, |profiler| {
            let event_label = profiler.get_or_alloc_cached_string(event_label);
            let event_id = EventId::from_label(event_label);
            TimingGuard::start(profiler, profiler.generic_activity_event_kind, event_id)
        })
    }

    /// Start profiling a generic activity. Profiling continues until the
    /// TimingGuard returned from this call is dropped.
    #[inline(always)]
    pub fn generic_activity_with_arg<A>(
        &self,
        event_label: &'static str,
        event_arg: A,
    ) -> TimingGuard<'_>
    where
        A: Borrow<str> + Into<String>,
    {
        self.exec(EventFilter::GENERIC_ACTIVITIES, |profiler| {
            let builder = EventIdBuilder::new(&profiler.profiler);
            let event_label = profiler.get_or_alloc_cached_string(event_label);
            let event_id = if profiler.event_filter_mask.contains(EventFilter::FUNCTION_ARGS) {
                let event_arg = profiler.get_or_alloc_cached_string(event_arg);
                builder.from_label_and_arg(event_label, event_arg)
            } else {
                builder.from_label(event_label)
            };
            TimingGuard::start(profiler, profiler.generic_activity_event_kind, event_id)
        })
    }

    /// Start profiling a query provider. Profiling continues until the
    /// TimingGuard returned from this call is dropped.
    #[inline(always)]
    pub fn query_provider(&self) -> TimingGuard<'_> {
        self.exec(EventFilter::QUERY_PROVIDERS, |profiler| {
            TimingGuard::start(profiler, profiler.query_event_kind, EventId::INVALID)
        })
    }

    /// Record a query in-memory cache hit.
    #[inline(always)]
    pub fn query_cache_hit(&self, query_invocation_id: QueryInvocationId) {
        self.instant_query_event(
            |profiler| profiler.query_cache_hit_event_kind,
            query_invocation_id,
            EventFilter::QUERY_CACHE_HITS,
        );
    }

    /// Start profiling a query being blocked on a concurrent execution.
    /// Profiling continues until the TimingGuard returned from this call is
    /// dropped.
    #[inline(always)]
    pub fn query_blocked(&self) -> TimingGuard<'_> {
        self.exec(EventFilter::QUERY_BLOCKED, |profiler| {
            TimingGuard::start(profiler, profiler.query_blocked_event_kind, EventId::INVALID)
        })
    }

    /// Start profiling how long it takes to load a query result from the
    /// incremental compilation on-disk cache. Profiling continues until the
    /// TimingGuard returned from this call is dropped.
    #[inline(always)]
    pub fn incr_cache_loading(&self) -> TimingGuard<'_> {
        self.exec(EventFilter::INCR_CACHE_LOADS, |profiler| {
            TimingGuard::start(
                profiler,
                profiler.incremental_load_result_event_kind,
                EventId::INVALID,
            )
        })
    }

    #[inline(always)]
    fn instant_query_event(
        &self,
        event_kind: fn(&SelfProfiler) -> StringId,
        query_invocation_id: QueryInvocationId,
        event_filter: EventFilter,
    ) {
        drop(self.exec(event_filter, |profiler| {
            let event_id = StringId::new_virtual(query_invocation_id.0);
            let thread_id = std::thread::current().id().as_u64().get() as u32;

            profiler.profiler.record_instant_event(
                event_kind(profiler),
                EventId::from_virtual(event_id),
                thread_id,
            );

            TimingGuard::none()
        }));
    }

    pub fn with_profiler(&self, f: impl FnOnce(&SelfProfiler)) {
        if let Some(profiler) = &self.profiler {
            f(&profiler)
        }
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.profiler.is_some()
    }

    #[inline]
    pub fn llvm_recording_enabled(&self) -> bool {
        self.event_filter_mask.contains(EventFilter::LLVM)
    }
    #[inline]
    pub fn get_self_profiler(&self) -> Option<Arc<SelfProfiler>> {
        self.profiler.clone()
    }
}

pub struct SelfProfiler {
    profiler: Profiler,
    event_filter_mask: EventFilter,

    string_cache: RwLock<FxHashMap<String, StringId>>,

    query_event_kind: StringId,
    generic_activity_event_kind: StringId,
    incremental_load_result_event_kind: StringId,
    query_blocked_event_kind: StringId,
    query_cache_hit_event_kind: StringId,
}

impl SelfProfiler {
    pub fn new(
        output_directory: &Path,
        crate_name: Option<&str>,
        event_filters: &Option<Vec<String>>,
    ) -> Result<SelfProfiler, Box<dyn Error>> {
        fs::create_dir_all(output_directory)?;

        let crate_name = crate_name.unwrap_or("unknown-crate");
        let filename = format!("{}-{}.rustc_profile", crate_name, process::id());
        let path = output_directory.join(&filename);
        let profiler = Profiler::new(&path)?;

        let query_event_kind = profiler.alloc_string("Query");
        let generic_activity_event_kind = profiler.alloc_string("GenericActivity");
        let incremental_load_result_event_kind = profiler.alloc_string("IncrementalLoadResult");
        let query_blocked_event_kind = profiler.alloc_string("QueryBlocked");
        let query_cache_hit_event_kind = profiler.alloc_string("QueryCacheHit");

        let mut event_filter_mask = EventFilter::empty();

        if let Some(ref event_filters) = *event_filters {
            let mut unknown_events = vec![];
            for item in event_filters {
                if let Some(&(_, mask)) =
                    EVENT_FILTERS_BY_NAME.iter().find(|&(name, _)| name == item)
                {
                    event_filter_mask |= mask;
                } else {
                    unknown_events.push(item.clone());
                }
            }

            // Warn about any unknown event names
            if !unknown_events.is_empty() {
                unknown_events.sort();
                unknown_events.dedup();

                warn!(
                    "Unknown self-profiler events specified: {}. Available options are: {}.",
                    unknown_events.join(", "),
                    EVENT_FILTERS_BY_NAME
                        .iter()
                        .map(|&(name, _)| name.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        } else {
            event_filter_mask = EventFilter::DEFAULT;
        }

        Ok(SelfProfiler {
            profiler,
            event_filter_mask,
            string_cache: RwLock::new(FxHashMap::default()),
            query_event_kind,
            generic_activity_event_kind,
            incremental_load_result_event_kind,
            query_blocked_event_kind,
            query_cache_hit_event_kind,
        })
    }

    /// Allocates a new string in the profiling data. Does not do any caching
    /// or deduplication.
    pub fn alloc_string<STR: SerializableString + ?Sized>(&self, s: &STR) -> StringId {
        self.profiler.alloc_string(s)
    }

    /// Gets a `StringId` for the given string. This method makes sure that
    /// any strings going through it will only be allocated once in the
    /// profiling data.
    pub fn get_or_alloc_cached_string<A>(&self, s: A) -> StringId
    where
        A: Borrow<str> + Into<String>,
    {
        // Only acquire a read-lock first since we assume that the string is
        // already present in the common case.
        {
            let string_cache = self.string_cache.read();

            if let Some(&id) = string_cache.get(s.borrow()) {
                return id;
            }
        }

        let mut string_cache = self.string_cache.write();
        // Check if the string has already been added in the small time window
        // between dropping the read lock and acquiring the write lock.
        match string_cache.entry(s.into()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let string_id = self.profiler.alloc_string(&e.key()[..]);
                *e.insert(string_id)
            }
        }
    }

    pub fn map_query_invocation_id_to_string(&self, from: QueryInvocationId, to: StringId) {
        let from = StringId::new_virtual(from.0);
        self.profiler.map_virtual_to_concrete_string(from, to);
    }

    pub fn bulk_map_query_invocation_id_to_single_string<I>(&self, from: I, to: StringId)
    where
        I: Iterator<Item = QueryInvocationId> + ExactSizeIterator,
    {
        let from = from.map(|qid| StringId::new_virtual(qid.0));
        self.profiler.bulk_map_virtual_to_single_concrete_string(from, to);
    }

    pub fn query_key_recording_enabled(&self) -> bool {
        self.event_filter_mask.contains(EventFilter::QUERY_KEYS)
    }

    pub fn event_id_builder(&self) -> EventIdBuilder<'_, SerializationSink> {
        EventIdBuilder::new(&self.profiler)
    }
}

#[must_use]
pub struct TimingGuard<'a>(Option<measureme::TimingGuard<'a, SerializationSink>>);

impl<'a> TimingGuard<'a> {
    #[inline]
    pub fn start(
        profiler: &'a SelfProfiler,
        event_kind: StringId,
        event_id: EventId,
    ) -> TimingGuard<'a> {
        let thread_id = std::thread::current().id().as_u64().get() as u32;
        let raw_profiler = &profiler.profiler;
        let timing_guard =
            raw_profiler.start_recording_interval_event(event_kind, event_id, thread_id);
        TimingGuard(Some(timing_guard))
    }

    #[inline]
    pub fn finish_with_query_invocation_id(self, query_invocation_id: QueryInvocationId) {
        if let Some(guard) = self.0 {
            cold_path(|| {
                let event_id = StringId::new_virtual(query_invocation_id.0);
                let event_id = EventId::from_virtual(event_id);
                guard.finish_with_override_event_id(event_id);
            });
        }
    }

    #[inline]
    pub fn none() -> TimingGuard<'a> {
        TimingGuard(None)
    }

    #[inline(always)]
    pub fn run<R>(self, f: impl FnOnce() -> R) -> R {
        let _timer = self;
        f()
    }
}

#[must_use]
pub struct VerboseTimingGuard<'a> {
    start_and_message: Option<(Instant, String)>,
    _guard: TimingGuard<'a>,
}

impl<'a> VerboseTimingGuard<'a> {
    pub fn start(message: Option<String>, _guard: TimingGuard<'a>) -> Self {
        VerboseTimingGuard { _guard, start_and_message: message.map(|msg| (Instant::now(), msg)) }
    }

    #[inline(always)]
    pub fn run<R>(self, f: impl FnOnce() -> R) -> R {
        let _timer = self;
        f()
    }
}

impl Drop for VerboseTimingGuard<'_> {
    fn drop(&mut self) {
        if let Some((start, ref message)) = self.start_and_message {
            print_time_passes_entry(true, &message[..], start.elapsed());
        }
    }
}

pub fn print_time_passes_entry(do_it: bool, what: &str, dur: Duration) {
    if !do_it {
        return;
    }

    let mem_string = match get_resident() {
        Some(n) => {
            let mb = n as f64 / 1_000_000.0;
            format!("; rss: {}MB", mb.round() as usize)
        }
        None => String::new(),
    };
    println!("time: {}{}\t{}", duration_to_secs_str(dur), mem_string, what);
}

// Hack up our own formatting for the duration to make it easier for scripts
// to parse (always use the same number of decimal places and the same unit).
pub fn duration_to_secs_str(dur: std::time::Duration) -> String {
    const NANOS_PER_SEC: f64 = 1_000_000_000.0;
    let secs = dur.as_secs() as f64 + dur.subsec_nanos() as f64 / NANOS_PER_SEC;

    format!("{:.3}", secs)
}

// Memory reporting
cfg_if! {
    if #[cfg(windows)] {
        fn get_resident() -> Option<usize> {
            use std::mem::{self, MaybeUninit};
            use winapi::shared::minwindef::DWORD;
            use winapi::um::processthreadsapi::GetCurrentProcess;
            use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

            let mut pmc = MaybeUninit::<PROCESS_MEMORY_COUNTERS>::uninit();
            match unsafe {
                GetProcessMemoryInfo(GetCurrentProcess(), pmc.as_mut_ptr(), mem::size_of_val(&pmc) as DWORD)
            } {
                0 => None,
                _ => {
                    let pmc = unsafe { pmc.assume_init() };
                    Some(pmc.WorkingSetSize as usize)
                }
            }
        }
    } else if #[cfg(unix)] {
        fn get_resident() -> Option<usize> {
            let field = 1;
            let contents = fs::read("/proc/self/statm").ok()?;
            let contents = String::from_utf8(contents).ok()?;
            let s = contents.split_whitespace().nth(field)?;
            let npages = s.parse::<usize>().ok()?;
            Some(npages * 4096)
        }
    } else {
        fn get_resident() -> Option<usize> {
            None
        }
    }
}
