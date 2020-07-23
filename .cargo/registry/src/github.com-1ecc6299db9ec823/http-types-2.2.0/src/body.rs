use async_std::io::prelude::*;
use async_std::io::{self, Cursor};
use serde::{de::DeserializeOwned, Serialize};

use std::fmt::{self, Debug};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{mime, Mime};
use crate::{Status, StatusCode};

pin_project_lite::pin_project! {
    /// A streaming HTTP body.
    ///
    /// `Body` represents the HTTP body of both `Request` and `Response`. It's completely
    /// streaming, and implements `AsyncBufRead` to make reading from it both convenient and
    /// performant.
    ///
    /// Both `Request` and `Response` take `Body` by `Into<Body>`, which means that passing string
    /// literals, byte vectors, but also concrete `Body` instances are all valid. This makes it
    /// easy to create both quick HTTP requests, but also have fine grained control over how bodies
    /// are streamed out.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Body, Response, StatusCode};
    /// use async_std::io::Cursor;
    ///
    /// let mut req = Response::new(StatusCode::Ok);
    /// req.set_body("Hello Chashu");
    ///
    /// let mut req = Response::new(StatusCode::Ok);
    /// let cursor = Cursor::new("Hello Nori");
    /// let body = Body::from_reader(cursor, Some(10)); // set the body length
    /// req.set_body(body);
    /// ```
    ///
    /// # Length
    ///
    /// One of the details of `Body` to be aware of is the `length` parameter. The value of
    /// `length` is used by HTTP implementations to determine how to treat the stream. If a length
    /// is known ahead of time, it's _strongly_ recommended to pass it.
    ///
    /// Casting from `Vec<u8>`, `String`, or similar to `Body` will automatically set the value of
    /// `length`.
    ///
    /// # Content Encoding
    ///
    /// By default `Body` will come with a fallback Mime type that is used by `Request` and
    /// `Response` if no other type has been set, and no other Mime type can be inferred.
    ///
    /// It's _strongly_ recommended to always set a mime type on both the `Request` and `Response`,
    /// and not rely on the fallback mechanisms. However, they're still there if you need them.
    pub struct Body {
        #[pin]
        reader: Box<dyn BufRead + Unpin + Send + Sync + 'static>,
        mime: Mime,
        length: Option<usize>,
    }
}

impl Body {
    /// Create a new empty `Body`.
    ///
    /// The body will have a length of `0`, and the Mime type set to `application/octet-stream` if
    /// no other mime type has been set or can be sniffed.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Body, Response, StatusCode};
    ///
    /// let mut req = Response::new(StatusCode::Ok);
    /// req.set_body(Body::empty());
    /// ```
    pub fn empty() -> Self {
        Self {
            reader: Box::new(io::empty()),
            mime: mime::BYTE_STREAM,
            length: Some(0),
        }
    }

    /// Create a `Body` from a reader with an optional length.
    ///
    /// The Mime type is set to `application/octet-stream` if no other mime type has been set or can
    /// be sniffed. If a `Body` has no length, HTTP implementations will often switch over to
    /// framed messages such as [Chunked
    /// Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding).
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Body, Response, StatusCode};
    /// use async_std::io::Cursor;
    ///
    /// let mut req = Response::new(StatusCode::Ok);
    ///
    /// let cursor = Cursor::new("Hello Nori");
    /// let len = 10;
    /// req.set_body(Body::from_reader(cursor, Some(len)));
    /// ```
    pub fn from_reader(
        reader: impl BufRead + Unpin + Send + Sync + 'static,
        len: Option<usize>,
    ) -> Self {
        Self {
            reader: Box::new(reader),
            mime: mime::BYTE_STREAM,
            length: len,
        }
    }

    /// Get the inner reader from the `Body`
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::prelude::*;
    /// use http_types::Body;
    /// use async_std::io::Cursor;
    ///
    /// let cursor = Cursor::new("Hello Nori");
    /// let body = Body::from_reader(cursor, None);
    /// let _ = body.into_reader();
    /// ```
    pub fn into_reader(self) -> Box<dyn BufRead + Unpin + Send + 'static> {
        self.reader
    }

    /// Create a `Body` from a Vec of bytes.
    ///
    /// The Mime type is set to `application/octet-stream` if no other mime type has been set or can
    /// be sniffed. If a `Body` has no length, HTTP implementations will often switch over to
    /// framed messages such as [Chunked
    /// Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding).
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Body, Response, StatusCode};
    /// use async_std::io::Cursor;
    ///
    /// let mut req = Response::new(StatusCode::Ok);
    ///
    /// let input = vec![1, 2, 3];
    /// req.set_body(Body::from_bytes(input));
    /// ```
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            mime: mime::BYTE_STREAM,
            length: Some(bytes.len()),
            reader: Box::new(io::Cursor::new(bytes)),
        }
    }

    /// Parse the body into a `Vec<u8>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::Body;
    ///
    /// let bytes = vec![1, 2, 3];
    /// let body = Body::from_bytes(bytes);
    ///
    /// let bytes: Vec<u8> = body.into_bytes().await?;
    /// assert_eq!(bytes, vec![1, 2, 3]);
    /// # Ok(()) }) }
    /// ```
    pub async fn into_bytes(mut self) -> crate::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(1024);
        self.read_to_end(&mut buf)
            .await
            .status(StatusCode::UnprocessableEntity)?;
        Ok(buf)
    }

    /// Create a `Body` from a String
    ///
    /// The Mime type is set to `text/plain` if no other mime type has been set or can
    /// be sniffed. If a `Body` has no length, HTTP implementations will often switch over to
    /// framed messages such as [Chunked
    /// Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding).
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Body, Response, StatusCode};
    /// use async_std::io::Cursor;
    ///
    /// let mut req = Response::new(StatusCode::Ok);
    ///
    /// let input = String::from("hello Nori!");
    /// req.set_body(Body::from_string(input));
    /// ```
    pub fn from_string(s: String) -> Self {
        Self {
            mime: mime::PLAIN,
            length: Some(s.len()),
            reader: Box::new(io::Cursor::new(s.into_bytes())),
        }
    }

    /// Read the body as a string
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// # async_std::task::block_on(async {
    /// use http_types::Body;
    /// use async_std::io::Cursor;
    ///  
    /// let cursor = Cursor::new("Hello Nori");
    /// let body = Body::from_reader(cursor, None);
    /// assert_eq!(&body.into_string().await.unwrap(), "Hello Nori");
    /// # Ok(()) }) }
    /// ```
    pub async fn into_string(mut self) -> crate::Result<String> {
        let mut result = String::with_capacity(self.len().unwrap_or(0));
        self.read_to_string(&mut result)
            .await
            .status(StatusCode::UnprocessableEntity)?;
        Ok(result)
    }

    /// Creates a `Body` from a type, serializing it as JSON.
    ///
    /// # Mime
    ///
    /// The encoding is set to `application/json`.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Body, convert::json};
    ///
    /// let body = Body::from_json(&json!({ "name": "Chashu" }));
    /// # drop(body);
    /// ```
    pub fn from_json(json: &impl Serialize) -> crate::Result<Self> {
        let bytes = serde_json::to_vec(&json)?;
        let body = Self {
            length: Some(bytes.len()),
            reader: Box::new(Cursor::new(bytes)),
            mime: mime::JSON,
        };
        Ok(body)
    }

    /// Parse the body as JSON, serializing it to a struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::Body;
    /// use http_types::convert::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Cat { name: String }
    ///
    /// let cat = Cat { name: String::from("chashu") };
    /// let body = Body::from_json(&cat)?;
    ///
    /// let cat: Cat = body.into_json().await?;
    /// assert_eq!(&cat.name, "chashu");
    /// # Ok(()) }) }
    /// ```
    pub async fn into_json<T: DeserializeOwned>(mut self) -> crate::Result<T> {
        let mut buf = Vec::with_capacity(1024);
        self.read_to_end(&mut buf).await?;
        Ok(serde_json::from_slice(&buf).status(StatusCode::UnprocessableEntity)?)
    }

    /// Creates a `Body` from a type, serializing it using form encoding.
    ///
    /// # Mime
    ///
    /// The encoding is set to `application/x-www-form-urlencoded`.
    ///
    /// # Errors
    ///
    /// An error will be returned if the encoding failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::Body;
    /// use http_types::convert::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Cat { name: String }
    ///
    /// let cat = Cat { name: String::from("chashu") };
    /// let body = Body::from_form(&cat)?;
    ///
    /// let cat: Cat = body.into_form().await?;
    /// assert_eq!(&cat.name, "chashu");
    /// # Ok(()) }) }
    /// ```
    pub fn from_form(form: &impl Serialize) -> crate::Result<Self> {
        let query = serde_urlencoded::to_string(form)?;
        let bytes = query.into_bytes();

        let body = Self {
            length: Some(bytes.len()),
            reader: Box::new(Cursor::new(bytes)),
            mime: mime::FORM,
        };
        Ok(body)
    }

    /// Parse the body from form encoding into a type.
    ///
    /// # Errors
    ///
    /// An error is returned if the underlying IO stream errors, or if the body
    /// could not be deserialized into the type.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::Body;
    /// use http_types::convert::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Cat { name: String }
    ///
    /// let cat = Cat { name: String::from("chashu") };
    /// let body = Body::from_form(&cat)?;
    ///
    /// let cat: Cat = body.into_form().await?;
    /// assert_eq!(&cat.name, "chashu");
    /// # Ok(()) }) }
    /// ```
    pub async fn into_form<T: DeserializeOwned>(self) -> crate::Result<T> {
        let s = self.into_string().await?;
        Ok(serde_urlencoded::from_str(&s).status(StatusCode::UnprocessableEntity)?)
    }

    /// Create a `Body` from a file.
    ///
    /// The Mime type set to `application/octet-stream` if no other mime type has
    /// been set or can be sniffed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::{Body, Response, StatusCode};
    ///
    /// let mut res = Response::new(StatusCode::Ok);
    /// res.set_body(Body::from_file("/path/to/file").await?);
    /// # Ok(()) }) }
    /// ```
    #[cfg(all(feature = "async_std", not(target_os = "unknown")))]
    pub async fn from_file<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();
        let mut file = async_std::fs::File::open(path).await?;
        let len = file.metadata().await?.len();

        // Look at magic bytes first, look at extension second, fall back to
        // octet stream.
        let mime = peek_mime(&mut file)
            .await?
            .or_else(|| guess_ext(path))
            .unwrap_or(mime::BYTE_STREAM);

        Ok(Self {
            mime,
            length: Some(len as usize),
            reader: Box::new(io::BufReader::new(file)),
        })
    }

    /// Get the length of the body in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::Body;
    /// use async_std::io::Cursor;
    ///
    /// let cursor = Cursor::new("Hello Nori");
    /// let len = 10;
    /// let body = Body::from_reader(cursor, Some(len));
    /// assert_eq!(body.len(), Some(10));
    /// ```
    pub fn len(&self) -> Option<usize> {
        self.length
    }

    /// Returns `true` if the body has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> Option<bool> {
        self.length.map(|length| length == 0)
    }

    pub(crate) fn mime(&self) -> &Mime {
        &self.mime
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body")
            .field("reader", &"<hidden>")
            .field("length", &self.length)
            .finish()
    }
}

impl From<serde_json::Value> for Body {
    fn from(json_value: serde_json::Value) -> Self {
        Self::from_json(&json_value).unwrap()
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl<'a> From<&'a str> for Body {
    fn from(s: &'a str) -> Self {
        Self::from_string(s.to_owned())
    }
}

impl From<Vec<u8>> for Body {
    fn from(b: Vec<u8>) -> Self {
        Self::from_bytes(b.to_owned())
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(b: &'a [u8]) -> Self {
        Self::from_bytes(b.to_owned())
    }
}

impl Read for Body {
    #[allow(missing_doc_code_examples)]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

impl BufRead for Body {
    #[allow(missing_doc_code_examples)]
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&'_ [u8]>> {
        let this = self.project();
        this.reader.poll_fill_buf(cx)
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.reader).consume(amt)
    }
}

/// Look at first few bytes of a file to determine the mime type.
/// This is used for various binary formats such as images and videos.
#[cfg(all(feature = "async_std", not(target_os = "unknown")))]
async fn peek_mime(file: &mut async_std::fs::File) -> io::Result<Option<Mime>> {
    // We need to read the first 300 bytes to correctly infer formats such as tar.
    let mut buf = [0_u8; 300];
    file.read(&mut buf).await?;
    let mime = Mime::sniff(&buf).ok();

    // Reset the file cursor back to the start.
    file.seek(io::SeekFrom::Start(0)).await?;
    Ok(mime)
}

/// Look at the extension of a file to determine the mime type.
/// This is useful for plain-text formats such as HTML and CSS.
#[cfg(all(feature = "async_std", not(target_os = "unknown")))]
fn guess_ext(path: &std::path::Path) -> Option<Mime> {
    let ext = path.extension().map(|p| p.to_str()).flatten();
    match ext {
        Some("html") => Some(mime::HTML),
        Some("js") | Some("mjs") | Some("jsonp") => Some(mime::JAVASCRIPT),
        Some("json") => Some(mime::JSON),
        Some("css") => Some(mime::CSS),
        Some("svg") => Some(mime::SVG),
        None | Some(_) => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[async_std::test]
    async fn json_status() {
        #[derive(Debug, Deserialize)]
        struct Foo {
            inner: String,
        }
        let body = Body::empty();
        let res = body.into_json::<Foo>().await;
        assert_eq!(res.unwrap_err().status(), 422);
    }

    #[async_std::test]
    async fn form_status() {
        #[derive(Debug, Deserialize)]
        struct Foo {
            inner: String,
        }
        let body = Body::empty();
        let res = body.into_form::<Foo>().await;
        assert_eq!(res.unwrap_err().status(), 422);
    }
}
