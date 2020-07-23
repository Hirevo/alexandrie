use crate::Message;

use std::time::Duration;

/// The kind of SSE event sent.
#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    /// A retry frame, signaling a new retry duration must be used..
    Retry(Duration),
    /// A data frame containing a message.
    Message(Message),
}

impl Event {
    /// Create a new message.
    pub(crate) fn new_msg(name: String, data: Vec<u8>, id: Option<String>) -> Self {
        Self::Message(Message { name, data, id })
    }

    /// Create a new retry.
    pub(crate) fn new_retry(dur: u64) -> Self {
        Self::Retry(Duration::from_secs_f64(dur as f64))
    }

    /// Check whether this is a Retry variant.
    pub fn is_retry(&self) -> bool {
        match *self {
            Self::Retry(_) => true,
            _ => false,
        }
    }

    /// Check whether this is a `Message` variant.
    pub fn is_message(&self) -> bool {
        match *self {
            Self::Message(_) => true,
            _ => false,
        }
    }
}
