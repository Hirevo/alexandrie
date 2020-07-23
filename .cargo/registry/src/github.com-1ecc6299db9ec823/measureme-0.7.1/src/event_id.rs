use crate::{Profiler, SerializationSink, StringComponent, StringId};

/// Event IDs are strings conforming to the following grammar:
///
/// ```ignore
///   <event_id> = <label> {<argument>}
///   <label> = <text>
///   <argument> = '\x1E' <text>
///   <text> = regex([^[[:cntrl:]]]+) // Anything but ASCII control characters
///  ```
///
/// This means there's always a "label", followed by an optional list of
/// arguments. Future versions my support other optional suffixes (with a tag
/// other than '\x11' after the '\x1E' separator), such as a "category".

/// The byte used to separate arguments from the label and each other.
pub const SEPARATOR_BYTE: &str = "\x1E";

/// An `EventId` is a `StringId` with the additional guarantee that the
/// corresponding string conforms to the event_id grammar.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
#[repr(C)]
pub struct EventId(StringId);

impl EventId {
    pub const INVALID: EventId = EventId(StringId::INVALID);

    #[inline]
    pub fn to_string_id(self) -> StringId {
        self.0
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0.as_u32()
    }

    #[inline]
    pub fn from_label(label: StringId) -> EventId {
        EventId(label)
    }

    #[inline]
    pub fn from_virtual(virtual_id: StringId) -> EventId {
        EventId(virtual_id)
    }

    /// Create an EventId from a raw u32 value. Only used internally for
    /// deserialization.
    #[inline]
    pub fn from_u32(raw_id: u32) -> EventId {
        EventId(StringId::new(raw_id))
    }
}

pub struct EventIdBuilder<'p, S: SerializationSink> {
    profiler: &'p Profiler<S>,
}

impl<'p, S: SerializationSink> EventIdBuilder<'p, S> {
    pub fn new(profiler: &Profiler<S>) -> EventIdBuilder<'_, S> {
        EventIdBuilder { profiler }
    }

    #[inline]
    pub fn from_label(&self, label: StringId) -> EventId {
        // Just forward the string ID, a single identifier is a valid event_id
        EventId::from_label(label)
    }

    pub fn from_label_and_arg(&self, label: StringId, arg: StringId) -> EventId {
        EventId(self.profiler.alloc_string(&[
            // Label
            StringComponent::Ref(label),
            // Seperator and start tag for arg
            StringComponent::Value(SEPARATOR_BYTE),
            // Arg string id
            StringComponent::Ref(arg),
        ]))
    }
}
