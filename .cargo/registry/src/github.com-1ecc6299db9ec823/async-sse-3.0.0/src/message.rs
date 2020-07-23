/// An SSE event with a data payload.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Message {
    /// The ID of this event.
    ///
    /// See also the [Server-Sent Events spec](https://html.spec.whatwg.org/multipage/server-sent-events.html#concept-event-stream-last-event-id).
    pub(crate) id: Option<String>,
    /// The event name. Defaults to "message" if no event name is provided.
    pub(crate) name: String,
    /// The data for this event.
    pub(crate) data: Vec<u8>,
}

impl Message {
    /// Get the message id.
    pub fn id(&self) -> &Option<String> {
        &self.id
    }

    /// Get the message event name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Access the event data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Convert the message into the data payload.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}
