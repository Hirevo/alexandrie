use async_std::io::{BufReader, Read};
use async_std::prelude::*;
use http_types::{ensure, ensure_eq, format_err};
use http_types::{
    headers::{CONTENT_LENGTH, DATE, TRANSFER_ENCODING},
    Body, Response, StatusCode,
};

use std::convert::TryFrom;

use crate::chunked::ChunkedDecoder;
use crate::date::fmt_http_date;
use crate::{MAX_HEADERS, MAX_HEAD_LENGTH};

const CR: u8 = b'\r';
const LF: u8 = b'\n';

/// Decode an HTTP response on the client.
#[doc(hidden)]
pub async fn decode<R>(reader: R) -> http_types::Result<Response>
where
    R: Read + Unpin + Send + Sync + 'static,
{
    let mut reader = BufReader::new(reader);
    let mut buf = Vec::new();
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut httparse_res = httparse::Response::new(&mut headers);

    // Keep reading bytes from the stream until we hit the end of the stream.
    loop {
        let bytes_read = reader.read_until(LF, &mut buf).await?;
        // No more bytes are yielded from the stream.
        assert!(bytes_read != 0, "Empty response"); // TODO: ensure?

        // Prevent CWE-400 DDOS with large HTTP Headers.
        ensure!(
            buf.len() < MAX_HEAD_LENGTH,
            "Head byte length should be less than 8kb"
        );

        // We've hit the end delimiter of the stream.
        let idx = buf.len() - 1;
        if idx >= 3 && &buf[idx - 3..=idx] == [CR, LF, CR, LF] {
            break;
        }
        if idx >= 1 && &buf[idx - 1..=idx] == [LF, LF] {
            break;
        }
    }

    // Convert our header buf into an httparse instance, and validate.
    let status = httparse_res.parse(&buf)?;
    ensure!(!status.is_partial(), "Malformed HTTP head");

    let code = httparse_res.code;
    let code = code.ok_or_else(|| format_err!("No status code found"))?;

    // Convert httparse headers + body into a `http_types::Response` type.
    let version = httparse_res.version;
    let version = version.ok_or_else(|| format_err!("No version found"))?;
    ensure_eq!(version, 1, "Unsupported HTTP version");

    let mut res = Response::new(StatusCode::try_from(code)?);
    for header in httparse_res.headers.iter() {
        res.append_header(header.name, std::str::from_utf8(header.value)?);
    }

    if res.header(DATE).is_none() {
        let date = fmt_http_date(std::time::SystemTime::now());
        res.insert_header(DATE, &format!("date: {}\r\n", date)[..]);
    }

    let content_length = res.header(CONTENT_LENGTH);
    let transfer_encoding = res.header(TRANSFER_ENCODING);

    ensure!(
        content_length.is_none() || transfer_encoding.is_none(),
        "Unexpected Content-Length header"
    );

    if let Some(encoding) = transfer_encoding {
        if encoding.last().as_str() == "chunked" {
            let trailers_sender = res.send_trailers();
            let reader = BufReader::new(ChunkedDecoder::new(reader, trailers_sender));
            res.set_body(Body::from_reader(reader, None));

            // Return the response.
            return Ok(res);
        }
    }

    // Check for Content-Length.
    if let Some(len) = content_length {
        let len = len.last().as_str().parse::<usize>()?;
        res.set_body(Body::from_reader(reader.take(len as u64), Some(len)));
    }

    // Return the response.
    Ok(res)
}
