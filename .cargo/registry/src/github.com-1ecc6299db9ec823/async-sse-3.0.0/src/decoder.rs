use async_std::io::BufRead as AsyncBufRead;
use async_std::stream::Stream;
use async_std::task::{self, Context, Poll};

use std::pin::Pin;

use crate::Event;
use crate::Lines;

/// Decode a new incoming SSE connection.
pub fn decode<R>(reader: R) -> Decoder<R>
where
    R: AsyncBufRead + Unpin,
{
    Decoder {
        lines: Lines::new(reader),
        processed_bom: false,
        buffer: vec![],
        last_event_id: None,
        event_type: None,
        data: vec![],
    }
}

/// An SSE protocol decoder.
#[derive(Debug)]
pub struct Decoder<R: AsyncBufRead + Unpin> {
    /// The lines decoder.
    lines: Lines<R>,
    /// Have we processed the optional Byte Order Marker on the first line?
    processed_bom: bool,
    /// Was the last character of the previous line a \r?
    /// Bytes that were fed to the decoder but do not yet form a message.
    buffer: Vec<u8>,
    /// The _last event ID_ buffer.
    last_event_id: Option<String>,
    /// The _event type_ buffer.
    event_type: Option<String>,
    /// The _data_ buffer.
    data: Vec<u8>,
}

impl<R: AsyncBufRead + Unpin> Decoder<R> {
    fn take_message(&mut self) -> Option<Event> {
        if self.data.is_empty() {
            // If the data buffer is an empty string, set the data buffer and
            // the event type buffer to the empty string [and return.]
            self.event_type.take();
            None
        } else {
            // Removing tailing newlines
            if self.data.ends_with(&[b'\n']) {
                self.data.pop();
            }
            let name = self.event_type.take().unwrap_or("message".to_string());
            let data = std::mem::replace(&mut self.data, vec![]);
            // The _last event ID_ buffer persists between messages.
            let id = self.last_event_id.clone();
            Some(Event::new_msg(name, data, id))
        }
    }
}

impl<R: AsyncBufRead + Unpin> Stream for Decoder<R> {
    type Item = http_types::Result<Event>;

    // This function uses two loops: one to get lines from the reader.
    // And one to parse each line delimited by `:`.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Get the next line, if available.
            let line = match task::ready!(Pin::new(&mut self.lines).poll_next(cx)) {
                None => return Poll::Ready(None),
                Some(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                Some(Ok(line)) => line,
            };

            // Get rid of the BOM at the start
            let line = if !self.processed_bom && line.starts_with("\u{feff}") {
                self.processed_bom = true;
                &line[3..]
            } else {
                &line
            };

            log::trace!("> new line: {:?}", line);
            let mut parts = line.splitn(2, ':');
            loop {
                match (parts.next(), parts.next()) {
                    // If the field name is "retry":
                    (Some("retry"), Some(value)) if value.chars().all(|c| c.is_ascii_digit()) => {
                        log::trace!("> retry");
                        // If the field value consists of only ASCII digits, then interpret the field value
                        // as an integer in base ten, and set the event stream's reconnection time to that
                        // integer. Otherwise, ignore the field.
                        if let Ok(time) = value.parse::<u64>() {
                            return Poll::Ready(Some(Ok(Event::new_retry(time))));
                        }
                    }
                    // If the field name is "event":
                    (Some("event"), Some(value)) => {
                        log::trace!("> event");
                        // Set the event type buffer to field value.
                        self.event_type = Some(strip_leading_space(value).to_string());
                    }
                    // If the field name is "data":
                    (Some("data"), value) => {
                        log::trace!("> data: {:?}", &value);
                        // Append the field value to the data buffer,
                        if let Some(value) = value {
                            self.data.extend(strip_leading_space_b(value.as_bytes()));
                            // then append a single U+000A LINE FEED (LF) character to the data buffer.
                        }
                        self.data.push(b'\n');
                    }
                    // If the field name is "id":
                    (Some("id"), Some(id_str)) if !id_str.contains(char::from(0)) => {
                        log::trace!("> id");
                        // If the field value does not contain U+0000 NULL, then set the last event ID buffer to the field value.
                        // Otherwise, ignore the field.
                        self.last_event_id = Some(strip_leading_space(id_str).to_string());
                        // return Poll::Ready(Ok(self.take_message()).transpose());
                    }
                    // Comment
                    (Some(""), Some(_)) => (log::trace!("> comment")),
                    // End of frame
                    (Some(""), None) => {
                        log::trace!("> end of frame");
                        match self.take_message() {
                            Some(event) => {
                                log::trace!("> end of frame [event]: {:?}", event);
                                return Poll::Ready(Some(Ok(event)));
                            }
                            None => {
                                log::trace!("> end of frame, break");
                                break;
                            }
                        };
                    }
                    (_, _) => {
                        break;
                    }
                };
            }
        }
    }
}

/// Remove a leading space (code point 0x20) from a string slice.
fn strip_leading_space(input: &str) -> &str {
    if input.starts_with(' ') {
        &input[1..]
    } else {
        input
    }
}

fn strip_leading_space_b(input: &[u8]) -> &[u8] {
    if input.starts_with(&[b' ']) {
        &input[1..]
    } else {
        input
    }
}
