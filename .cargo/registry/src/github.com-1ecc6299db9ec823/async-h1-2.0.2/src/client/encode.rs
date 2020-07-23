use async_std::io::{self, Read};
use async_std::prelude::*;
use async_std::task::{Context, Poll};
use http_types::format_err;
use http_types::{headers::HOST, Method, Request};

use std::pin::Pin;

/// An HTTP encoder.
#[doc(hidden)]
#[derive(Debug)]
pub struct Encoder {
    /// Keep track how far we've indexed into the headers + body.
    cursor: usize,
    /// HTTP headers to be sent.
    headers: Vec<u8>,
    /// Check whether we're done sending headers.
    headers_done: bool,
    /// Request with the HTTP body to be sent.
    request: Request,
    /// Check whether we're done with the body.
    body_done: bool,
    /// Keep track of how many bytes have been read from the body stream.
    body_bytes_read: usize,
}

impl Encoder {
    /// Encode an HTTP request on the client.
    pub async fn encode(req: Request) -> http_types::Result<Self> {
        let mut buf: Vec<u8> = vec![];

        let mut url = req.url().path().to_owned();
        if let Some(fragment) = req.url().fragment() {
            url.push('#');
            url.push_str(fragment);
        }
        if let Some(query) = req.url().query() {
            url.push('?');
            url.push_str(query);
        }

        // A client sending a CONNECT request MUST consists of only the host
        // name and port number of the tunnel destination, separated by a colon.
        // See: https://tools.ietf.org/html/rfc7231#section-4.3.6
        if req.method() == Method::Connect {
            let host = req.url().host_str();
            let host = host.ok_or_else(|| format_err!("Missing hostname"))?;
            let port = req.url().port_or_known_default();
            let port = port.ok_or_else(|| format_err!("Missing port"))?;
            url = format!("{}:{}", host, port);
        }

        let val = format!("{} {} HTTP/1.1\r\n", req.method(), url);
        log::trace!("> {}", &val);
        buf.write_all(val.as_bytes()).await?;

        if req.header(HOST).is_none() {
            // Insert Host header
            // Insert host
            let host = req.url().host_str();
            let host = host.ok_or_else(|| format_err!("Missing hostname"))?;
            let val = if let Some(port) = req.url().port() {
                format!("host: {}:{}\r\n", host, port)
            } else {
                format!("host: {}\r\n", host)
            };

            log::trace!("> {}", &val);
            buf.write_all(val.as_bytes()).await?;
        }

        // Insert Proxy-Connection header when method is CONNECT
        if req.method() == Method::Connect {
            let val = "proxy-connection: keep-alive\r\n".to_owned();
            log::trace!("> {}", &val);
            buf.write_all(val.as_bytes()).await?;
        }

        // If the body isn't streaming, we can set the content-length ahead of time. Else we need to
        // send all items in chunks.
        if let Some(len) = req.len() {
            let val = format!("content-length: {}\r\n", len);
            log::trace!("> {}", &val);
            buf.write_all(val.as_bytes()).await?;
        } else {
            // write!(&mut buf, "Transfer-Encoding: chunked\r\n")?;
            panic!("chunked encoding is not implemented yet");
            // See: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding
            //      https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Trailer
        }

        for (header, values) in req.iter() {
            for value in values.iter() {
                let val = format!("{}: {}\r\n", header, value);
                log::trace!("> {}", &val);
                buf.write_all(val.as_bytes()).await?;
            }
        }

        buf.write_all(b"\r\n").await?;

        Ok(Self {
            request: req,
            headers: buf,
            cursor: 0,
            headers_done: false,
            body_done: false,
            body_bytes_read: 0,
        })
    }
}

impl Read for Encoder {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // Send the headers. As long as the headers aren't fully sent yet we
        // keep sending more of the headers.
        let mut bytes_read = 0;
        if !self.headers_done {
            let len = std::cmp::min(self.headers.len() - self.cursor, buf.len());
            let range = self.cursor..self.cursor + len;
            buf[0..len].copy_from_slice(&self.headers[range]);
            self.cursor += len;
            if self.cursor == self.headers.len() {
                self.headers_done = true;
            }
            bytes_read += len;
        }

        if !self.body_done {
            let inner_poll_result =
                Pin::new(&mut self.request).poll_read(cx, &mut buf[bytes_read..]);
            let n = match inner_poll_result {
                Poll::Ready(Ok(n)) => n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    if bytes_read == 0 {
                        return Poll::Pending;
                    } else {
                        return Poll::Ready(Ok(bytes_read as usize));
                    }
                }
            };
            bytes_read += n;
            self.body_bytes_read += n;
            if bytes_read == 0 {
                self.body_done = true;
            }
        }

        Poll::Ready(Ok(bytes_read as usize))
    }
}
