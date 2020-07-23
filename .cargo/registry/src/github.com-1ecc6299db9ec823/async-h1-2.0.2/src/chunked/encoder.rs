use std::pin::Pin;

use async_std::io;
use async_std::io::prelude::*;
use async_std::task::{Context, Poll};
use http_types::Response;

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const CRLF_LEN: usize = 2;

/// The encoder state.
#[derive(Debug)]
enum State {
    /// Starting state.
    Start,
    /// Streaming out chunks.
    EncodeChunks,
    /// No more chunks to stream, mark the end.
    EndOfChunks,
    /// Receiving trailers from a channel.
    ReceiveTrailers,
    /// Streaming out trailers, if we received any.
    EncodeTrailers,
    /// Writing out the final CRLF.
    EndOfStream,
    /// The stream has finished.
    End,
}

/// An encoder for chunked encoding.
#[derive(Debug)]
pub(crate) struct ChunkedEncoder {
    /// How many bytes we've written to the buffer so far.
    bytes_written: usize,
    /// The internal encoder state.
    state: State,
}

impl ChunkedEncoder {
    /// Create a new instance.
    pub(crate) fn new() -> Self {
        Self {
            state: State::Start,
            bytes_written: 0,
        }
    }

    /// Encode an AsyncBufRead using "chunked" framing. This is used for streams
    /// whose length is not known up front.
    ///
    /// # Format
    ///
    /// Each "chunk" uses the following encoding:
    ///
    /// ```txt
    /// 1. {byte length of `data` as hex}\r\n
    /// 2. {data}\r\n
    /// ```
    ///
    /// A chunk stream is finalized by appending the following:
    ///
    /// ```txt
    /// 1. 0\r\n
    /// 2. {trailing header}\r\n (can be repeated)
    /// 3. \r\n
    /// ```
    pub(crate) fn encode(
        &mut self,
        res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.bytes_written = 0;
        let res = self.run(res, cx, buf);
        log::trace!("ChunkedEncoder {} bytes written", self.bytes_written);
        res
    }

    /// Execute the right method for the current state.
    fn run(
        &mut self,
        res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.state {
            State::Start => self.dispatch(State::EncodeChunks, res, cx, buf),
            State::EncodeChunks => self.encode_chunks(res, cx, buf),
            State::EndOfChunks => self.encode_chunks_eos(res, cx, buf),
            State::ReceiveTrailers => self.receive_trailers(res, cx, buf),
            State::EncodeTrailers => self.encode_trailers(res, cx, buf),
            State::EndOfStream => self.encode_eos(res, cx, buf),
            State::End => Poll::Ready(Ok(self.bytes_written)),
        }
    }

    /// Switch the internal state to a new state.
    fn dispatch(
        &mut self,
        state: State,
        res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        use State::*;
        log::trace!("ChunkedEncoder state: {:?} -> {:?}", self.state, state);

        #[cfg(debug_assertions)]
        match self.state {
            Start => assert!(matches!(state, EncodeChunks)),
            EncodeChunks => assert!(matches!(state, EndOfChunks)),
            EndOfChunks => assert!(matches!(state, ReceiveTrailers)),
            ReceiveTrailers => assert!(matches!(state, EncodeTrailers | EndOfStream)),
            EncodeTrailers => assert!(matches!(state, EndOfStream)),
            EndOfStream => assert!(matches!(state, End)),
            End => panic!("No state transitions allowed after the ChunkedEncoder has ended"),
        }

        self.state = state;
        self.run(res, cx, buf)
    }

    /// Stream out data using chunked encoding.
    fn encode_chunks(
        &mut self,
        mut res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // Get bytes from the underlying stream. If the stream is not ready yet,
        // return the header bytes if we have any.
        let src = match Pin::new(&mut res).poll_fill_buf(cx) {
            Poll::Ready(Ok(n)) => n,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => match self.bytes_written {
                0 => return Poll::Pending,
                n => return Poll::Ready(Ok(n)),
            },
        };

        // If the stream doesn't have any more bytes left to read we're done
        // sending chunks and it's time to move on.
        if src.len() == 0 {
            return self.dispatch(State::EndOfChunks, res, cx, buf);
        }

        // Each chunk is prefixed with the length of the data in hex, then a
        // CRLF, then the content, then another CRLF. Calculate how many bytes
        // each part should be.
        let buf_len = buf.len().checked_sub(self.bytes_written).unwrap_or(0);
        let msg_len = src.len().min(buf_len);
        // Calculate the max char count encoding the `len_prefix` statement
        // as hex would take. This is done by rounding up `log16(amt + 1)`.
        let hex_len = ((msg_len + 1) as f64).log(16.0).ceil() as usize;
        let framing_len = hex_len + CRLF_LEN * 2;
        let buf_upper = buf_len.checked_sub(framing_len).unwrap_or(0);
        let msg_len = msg_len.min(buf_upper);
        let len_prefix = format!("{:X}", msg_len).into_bytes();

        // Request a new buf if the current buf is too small to write any data
        // into. Empty frames should only be sent to mark the end of a stream.
        if buf.len() <= framing_len {
            cx.waker().wake_by_ref();
            return Poll::Ready(Ok(self.bytes_written));
        }

        // Write our frame header to the buffer.
        let lower = self.bytes_written;
        let upper = self.bytes_written + len_prefix.len();
        buf[lower..upper].copy_from_slice(&len_prefix);
        buf[upper] = CR;
        buf[upper + 1] = LF;
        self.bytes_written += len_prefix.len() + 2;

        // Copy the bytes from our source into the output buffer.
        let lower = self.bytes_written;
        let upper = self.bytes_written + msg_len;
        buf[lower..upper].copy_from_slice(&src[0..msg_len]);
        Pin::new(&mut res).consume(msg_len);
        self.bytes_written += msg_len;

        // Finalize the chunk with a closing CRLF.
        let idx = self.bytes_written;
        buf[idx] = CR;
        buf[idx + 1] = LF;
        self.bytes_written += CRLF_LEN;

        // Finally return how many bytes we've written to the buffer.
        Poll::Ready(Ok(self.bytes_written))
    }

    fn encode_chunks_eos(
        &mut self,
        res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // Request a new buf if the current buf is too small to write into.
        if buf.len() < 3 {
            cx.waker().wake_by_ref();
            return Poll::Ready(Ok(self.bytes_written));
        }

        // Write out the final empty chunk
        let idx = self.bytes_written;
        buf[idx] = b'0';
        buf[idx + 1] = CR;
        buf[idx + 2] = LF;
        self.bytes_written += 1 + CRLF_LEN;

        self.dispatch(State::ReceiveTrailers, res, cx, buf)
    }

    /// Receive trailers sent to the response, and store them in an internal
    /// buffer.
    fn receive_trailers(
        &mut self,
        res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // TODO: actually wait for trailers to be received.
        self.dispatch(State::EncodeTrailers, res, cx, buf)
    }

    /// Send trailers to the buffer.
    fn encode_trailers(
        &mut self,
        res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // TODO: actually encode trailers here.
        self.dispatch(State::EndOfStream, res, cx, buf)
    }

    /// Encode the end of the stream.
    fn encode_eos(
        &mut self,
        res: &mut Response,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let idx = self.bytes_written;
        // Write the final CRLF
        buf[idx] = CR;
        buf[idx + 1] = LF;
        self.bytes_written += CRLF_LEN;
        self.dispatch(State::End, res, cx, buf)
    }
}
