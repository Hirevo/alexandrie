use async_std::io::{self, BufRead, Read};
use async_std::sync;

use std::convert::{Into, TryInto};
use std::mem;
use std::ops::Index;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::convert::{DeserializeOwned, Serialize};
use crate::headers::{
    self, HeaderName, HeaderValue, HeaderValues, Headers, Names, ToHeaderValues, Values,
    CONTENT_TYPE,
};
use crate::mime::Mime;
use crate::trailers::{self, Trailers};
use crate::{Body, Extensions, Method, StatusCode, Url, Version};

pin_project_lite::pin_project! {
    /// An HTTP request.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Url, Method, Request};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body("Hello, Nori!");
    /// ```
    #[derive(Debug)]
    pub struct Request {
        method: Method,
        url: Url,
        headers: Headers,
        version: Option<Version>,
        #[pin]
        body: Body,
        local_addr: Option<String>,
        peer_addr: Option<String>,
        ext: Extensions,
        trailers_sender: Option<sync::Sender<Trailers>>,
        trailers_receiver: Option<sync::Receiver<Trailers>>,
    }
}

impl Request {
    /// Create a new request.
    pub fn new<U>(method: Method, url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        let url = url.try_into().expect("Could not convert into a valid url");
        let (trailers_sender, trailers_receiver) = sync::channel(1);
        Self {
            method,
            url,
            headers: Headers::new(),
            version: None,
            body: Body::empty(),
            ext: Extensions::new(),
            peer_addr: None,
            local_addr: None,
            trailers_receiver: Some(trailers_receiver),
            trailers_sender: Some(trailers_sender),
        }
    }

    /// Sets a string representation of the peer address of this
    /// request. This might take the form of an ip/fqdn and port or a
    /// local socket address.
    pub fn set_peer_addr(&mut self, peer_addr: Option<impl std::string::ToString>) {
        self.peer_addr = peer_addr.map(|addr| addr.to_string());
    }

    /// Sets a string representation of the local address that this
    /// request was received on. This might take the form of an ip/fqdn and
    /// port, or a local socket address.
    pub fn set_local_addr(&mut self, local_addr: Option<impl std::string::ToString>) {
        self.local_addr = local_addr.map(|addr| addr.to_string());
    }

    /// Get the peer socket address for the underlying transport, if
    /// that information is available for this request.
    pub fn peer_addr(&self) -> Option<&str> {
        self.peer_addr.as_deref()
    }

    /// Get the local socket address for the underlying transport, if
    /// that information is available for this request.
    pub fn local_addr(&self) -> Option<&str> {
        self.local_addr.as_deref()
    }

    /// Get the remote address for this request.
    ///
    /// This is determined in the following priority:
    /// 1. `Forwarded` header `for` key
    /// 2. The first `X-Forwarded-For` header
    /// 3. Peer address of the transport
    pub fn remote(&self) -> Option<&str> {
        self.forwarded_for().or_else(|| self.peer_addr())
    }

    /// Get the destination host for this request.
    ///
    /// This is determined in the following priority:
    /// 1. `Forwarded` header `host` key
    /// 2. The first `X-Forwarded-Host` header
    /// 3. `Host` header
    /// 4. URL domain, if any
    pub fn host(&self) -> Option<&str> {
        self.forwarded_header_part("host")
            .or_else(|| {
                self.header("X-Forwarded-Host")
                    .and_then(|h| h.as_str().split(",").next())
            })
            .or_else(|| self.header(&headers::HOST).map(|h| h.as_str()))
            .or_else(|| self.url().host_str())
    }

    fn forwarded_header_part(&self, part: &str) -> Option<&str> {
        self.header("Forwarded").and_then(|header| {
            header.as_str().split(";").find_map(|key_equals_value| {
                let parts = key_equals_value.split("=").collect::<Vec<_>>();
                if parts.len() == 2 && parts[0].eq_ignore_ascii_case(part) {
                    Some(parts[1])
                } else {
                    None
                }
            })
        })
    }

    fn forwarded_for(&self) -> Option<&str> {
        self.forwarded_header_part("for").or_else(|| {
            self.header("X-Forwarded-For")
                .and_then(|header| header.as_str().split(",").next())
        })
    }

    /// Get the HTTP method
    pub fn method(&self) -> Method {
        self.method
    }

    /// Set the HTTP method.
    pub fn set_method(&mut self, method: Method) {
        self.method = method;
    }

    /// Get a reference to the url.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> {
    /// #
    /// use http_types::{Method, Request, Response, StatusCode, Url};
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// assert_eq!(req.url().scheme(), "https");
    /// #
    /// # Ok(()) }
    /// ```
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get a mutable reference to the url.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> {
    /// #
    /// use http_types::{Method, Request, Response, StatusCode, Url};
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// req.url_mut().set_scheme("http");
    /// assert_eq!(req.url().scheme(), "http");
    /// #
    /// # Ok(()) }
    /// ```
    pub fn url_mut(&mut self) -> &mut Url {
        &mut self.url
    }

    /// Set the request body.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body("Hello, Nori!");
    /// ```
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.replace_body(body);
    }

    /// Swaps the value of the body with another body, without deinitializing
    /// either one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use async_std::io::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// # async_std::task::block_on(async {
    /// #
    /// use http_types::{Body, Method, Request, Url};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body("Hello, Nori!");
    /// let mut body: Body = req.replace_body("Hello, Chashu!");
    ///
    /// let mut string = String::new();
    /// body.read_to_string(&mut string).await?;
    /// assert_eq!(&string, "Hello, Nori!");
    /// #
    /// # Ok(()) }) }
    /// ```
    pub fn replace_body(&mut self, body: impl Into<Body>) -> Body {
        let body = mem::replace(&mut self.body, body.into());
        self.copy_content_type_from_body();
        body
    }

    /// Replace the request body with a new body, and return the old body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use async_std::io::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// # async_std::task::block_on(async {
    /// #
    /// use http_types::{Body, Method, Request, Url};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body("Hello, Nori!");
    /// let mut body = "Hello, Chashu!".into();
    /// req.swap_body(&mut body);
    ///
    /// let mut string = String::new();
    /// body.read_to_string(&mut string).await?;
    /// assert_eq!(&string, "Hello, Nori!");
    /// #
    /// # Ok(()) }) }
    /// ```
    pub fn swap_body(&mut self, body: &mut Body) {
        mem::swap(&mut self.body, body);
        self.copy_content_type_from_body();
    }

    /// Take the request body, replacing it with an empty body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use async_std::io::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// # async_std::task::block_on(async {
    /// #
    /// use http_types::{Body, Method, Request, Url};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body("Hello, Nori!");
    /// let mut body: Body = req.take_body();
    ///
    /// let mut string = String::new();
    /// body.read_to_string(&mut string).await?;
    /// assert_eq!(&string, "Hello, Nori!");
    ///
    /// # let mut string = String::new();
    /// # req.read_to_string(&mut string).await?;
    /// # assert_eq!(&string, "");
    /// #
    /// # Ok(()) }) }
    /// ```
    pub fn take_body(&mut self) -> Body {
        self.replace_body(Body::empty())
    }

    /// Read the body as a string.
    ///
    /// This consumes the request. If you want to read the body without
    /// consuming the request, consider using the `take_body` method and
    /// then calling `Body::into_string` or using the Request's AsyncRead
    /// implementation to read the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// # async_std::task::block_on(async {
    /// use async_std::io::Cursor;
    /// use http_types::{Body, Method, Request, Url};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    ///
    /// let cursor = Cursor::new("Hello Nori");
    /// let body = Body::from_reader(cursor, None);
    /// req.set_body(body);
    /// assert_eq!(&req.body_string().await.unwrap(), "Hello Nori");
    /// # Ok(()) }) }
    /// ```
    pub async fn body_string(&mut self) -> crate::Result<String> {
        let body = self.take_body();
        body.into_string().await
    }

    /// Read the body as bytes.
    ///
    /// This consumes the `Request`. If you want to read the body without
    /// consuming the request, consider using the `take_body` method and
    /// then calling `Body::into_bytes` or using the Request's AsyncRead
    /// implementation to read the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::{Body, Method, Request, Url};
    ///
    /// let bytes = vec![1, 2, 3];
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body(Body::from_bytes(bytes));
    ///
    /// let bytes = req.body_bytes().await?;
    /// assert_eq!(bytes, vec![1, 2, 3]);
    /// # Ok(()) }) }
    /// ```
    pub async fn body_bytes(&mut self) -> crate::Result<Vec<u8>> {
        let body = self.take_body();
        body.into_bytes().await
    }

    /// Read the body as JSON.
    ///
    /// This consumes the request. If you want to read the body without
    /// consuming the request, consider using the `take_body` method and
    /// then calling `Body::into_json` or using the Request's AsyncRead
    /// implementation to read the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::convert::{Deserialize, Serialize};
    /// use http_types::{Body, Method, Request, Url};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Cat {
    ///     name: String,
    /// }
    ///
    /// let cat = Cat {
    ///     name: String::from("chashu"),
    /// };
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body(Body::from_json(&cat)?);
    ///
    /// let cat: Cat = req.body_json().await?;
    /// assert_eq!(&cat.name, "chashu");
    /// # Ok(()) }) }
    /// ```
    pub async fn body_json<T: DeserializeOwned>(&mut self) -> crate::Result<T> {
        let body = self.take_body();
        body.into_json().await
    }

    /// Read the body as `x-www-form-urlencoded`.
    ///
    /// This consumes the request. If you want to read the body without
    /// consuming the request, consider using the `take_body` method and
    /// then calling `Body::into_json` or using the Request's AsyncRead
    /// implementation to read the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> { async_std::task::block_on(async {
    /// use http_types::convert::{Deserialize, Serialize};
    /// use http_types::{Body, Method, Request, Url};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Cat {
    ///     name: String,
    /// }
    ///
    /// let cat = Cat {
    ///     name: String::from("chashu"),
    /// };
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.set_body(Body::from_form(&cat)?);
    ///
    /// let cat: Cat = req.body_form().await?;
    /// assert_eq!(&cat.name, "chashu");
    /// # Ok(()) }) }
    /// ```
    pub async fn body_form<T: DeserializeOwned>(&mut self) -> crate::Result<T> {
        let body = self.take_body();
        body.into_form().await
    }

    /// Get an HTTP header.
    pub fn header(&self, name: impl Into<HeaderName>) -> Option<&HeaderValues> {
        self.headers.get(name)
    }

    /// Get a mutable reference to a header.
    pub fn header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValues> {
        self.headers.get_mut(name.into())
    }

    /// Remove a header.
    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.headers.remove(name.into())
    }

    /// Set an HTTP header.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// #
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// req.insert_header("Content-Type", "text/plain");
    /// #
    /// # Ok(()) }
    /// ```
    pub fn insert_header(
        &mut self,
        name: impl Into<HeaderName>,
        values: impl ToHeaderValues,
    ) -> Option<HeaderValues> {
        self.headers.insert(name, values)
    }

    /// Append a header to the headers.
    ///
    /// Unlike `insert` this function will not override the contents of a
    /// header, but insert a header if there aren't any. Or else append to
    /// the existing list of headers.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// #
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// req.append_header("Content-Type", "text/plain");
    /// #
    /// # Ok(()) }
    /// ```
    pub fn append_header(&mut self, name: impl Into<HeaderName>, values: impl ToHeaderValues) {
        self.headers.append(name, values)
    }

    /// Set the response MIME.
    // TODO: return a parsed MIME
    pub fn set_content_type(&mut self, mime: Mime) -> Option<HeaderValues> {
        let value: HeaderValue = mime.into();

        // A Mime instance is guaranteed to be valid header name.
        self.insert_header(CONTENT_TYPE, value)
    }

    /// Copy MIME data from the body.
    fn copy_content_type_from_body(&mut self) {
        if self.header(CONTENT_TYPE).is_none() {
            self.set_content_type(self.body.mime().clone());
        }
    }

    /// Get the current content type
    pub fn content_type(&self) -> Option<Mime> {
        self.header(CONTENT_TYPE)?.last().as_str().parse().ok()
    }

    /// Get the length of the body stream, if it has been set.
    ///
    /// This value is set when passing a fixed-size object into as the body.
    /// E.g. a string, or a buffer. Consumers of this API should check this
    /// value to decide whether to use `Chunked` encoding, or set the
    /// response length.
    pub fn len(&self) -> Option<usize> {
        self.body.len()
    }

    /// Returns `true` if the request has a set body stream length of zero,
    /// `false` otherwise.
    pub fn is_empty(&self) -> Option<bool> {
        self.body.is_empty()
    }

    /// Get the HTTP version, if one has been set.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url, Version};
    ///
    /// # fn main() -> Result<(), http_types::Error> {
    /// #
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// assert_eq!(req.version(), None);
    ///
    /// req.set_version(Some(Version::Http2_0));
    /// assert_eq!(req.version(), Some(Version::Http2_0));
    /// #
    /// # Ok(()) }
    /// ```
    pub fn version(&self) -> Option<Version> {
        self.version
    }

    /// Set the HTTP version.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url, Version};
    ///
    /// # fn main() -> Result<(), http_types::Error> {
    /// #
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// req.set_version(Some(Version::Http2_0));
    /// #
    /// # Ok(()) }
    /// ```
    pub fn set_version(&mut self, version: Option<Version>) {
        self.version = version;
    }

    /// Sends trailers to the a receiver.
    pub fn send_trailers(&mut self) -> trailers::Sender {
        let sender = self
            .trailers_sender
            .take()
            .expect("Trailers sender can only be constructed once");
        trailers::Sender::new(sender)
    }

    /// Receive trailers from a sender.
    pub async fn recv_trailers(&mut self) -> trailers::Receiver {
        let receiver = self
            .trailers_receiver
            .take()
            .expect("Trailers receiver can only be constructed once");
        trailers::Receiver::new(receiver)
    }

    /// An iterator visiting all header pairs in arbitrary order.
    pub fn iter(&self) -> headers::Iter<'_> {
        self.headers.iter()
    }

    /// An iterator visiting all header pairs in arbitrary order, with mutable
    /// references to the values.
    pub fn iter_mut(&mut self) -> headers::IterMut<'_> {
        self.headers.iter_mut()
    }

    /// An iterator visiting all header names in arbitrary order.
    pub fn header_names(&self) -> Names<'_> {
        self.headers.names()
    }

    /// An iterator visiting all header values in arbitrary order.
    pub fn header_values(&self) -> Values<'_> {
        self.headers.values()
    }

    /// Returns a reference to the existing local state.
    pub fn ext(&self) -> &Extensions {
        &self.ext
    }

    /// Returns a mutuable reference to the existing local state.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), http_types::Error> {
    /// #
    /// use http_types::{Method, Request, Url, Version};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// req.ext_mut().insert("hello from the extension");
    /// assert_eq!(req.ext().get(), Some(&"hello from the extension"));
    /// #
    /// # Ok(()) }
    /// ```
    pub fn ext_mut(&mut self) -> &mut Extensions {
        &mut self.ext
    }

    /// Get the URL querystring.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[async_std::main]
    /// # async fn main() -> http_types::Result<()> {
    /// use http_types::convert::{Deserialize, Serialize};
    /// use http_types::{Method, Request, Url};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Index {
    ///     page: u32,
    /// }
    ///
    /// let req = Request::new(
    ///     Method::Get,
    ///     Url::parse("https://httpbin.org/get?page=2").unwrap(),
    /// );
    /// let Index { page } = req.query()?;
    /// assert_eq!(page, 2);
    /// # Ok(()) }
    /// ```
    pub fn query<T: serde::de::DeserializeOwned>(&self) -> crate::Result<T> {
        // Default to an empty query string if no query parameter has been specified.
        // This allows successful deserialisation of structs where all fields are optional
        // when none of those fields has actually been passed by the caller.
        let query = self.url().query().unwrap_or("");
        serde_urlencoded::from_str(query).map_err(|e| {
            // Return the displayable version of the deserialisation error to the caller
            // for easier debugging.
            crate::Error::from_str(StatusCode::BadRequest, format!("{}", e))
        })
    }

    /// Set the URL querystring.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[async_std::main]
    /// # async fn main() -> http_types::Result<()> {
    /// use http_types::convert::{Deserialize, Serialize};
    /// use http_types::{Method, Request, Url};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Index {
    ///     page: u32,
    /// }
    ///
    /// let query = Index { page: 2 };
    /// let mut req = Request::new(
    ///     Method::Get,
    ///     Url::parse("https://httpbin.org/get?page=2").unwrap(),
    /// );
    /// req.set_query(&query)?;
    /// assert_eq!(req.url().query(), Some("page=2"));
    /// assert_eq!(req.url().as_str(), "https://httpbin.org/get?page=2");
    /// # Ok(()) }
    /// ```
    pub fn set_query(&mut self, query: &(impl Serialize + ?Sized)) -> crate::Result<()> {
        let query = serde_urlencoded::to_string(query)?;
        self.url.set_query(Some(&query));
        Ok(())
    }

    /// Create a `GET` request.
    ///
    /// The `GET` method requests a representation of the specified resource.
    /// Requests using `GET` should only retrieve data.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::get(Url::parse("https://example.com").unwrap());
    /// req.set_body("Hello, Nori!");
    /// assert_eq!(req.method(), Method::Get);
    /// ```
    pub fn get<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Get, url)
    }

    /// Create a `HEAD` request.
    ///
    /// The `HEAD` method asks for a response identical to that of a `GET`
    /// request, but without the response body.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::head(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Head);
    /// ```
    pub fn head<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Head, url)
    }

    /// Create a `POST` request.
    ///
    /// The `POST` method is used to submit an entity to the specified resource,
    /// often causing a change in state or side effects on the server.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::post(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Post);
    /// ```
    pub fn post<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Post, url)
    }

    /// Create a `PUT` request.
    ///
    /// The `PUT` method replaces all current representations of the target
    /// resource with the request payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::put(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Put);
    /// ```
    pub fn put<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Put, url)
    }

    /// Create a `DELETE` request.
    ///
    /// The `DELETE` method deletes the specified resource.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::delete(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Delete);
    /// ```
    pub fn delete<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Delete, url)
    }

    /// Create a `CONNECT` request.
    ///
    /// The `CONNECT` method establishes a tunnel to the server identified by
    /// the target resource.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::connect(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Connect);
    /// ```
    pub fn connect<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Connect, url)
    }

    /// Create a `OPTIONS` request.
    ///
    /// The `OPTIONS` method is used to describe the communication options for
    /// the target resource.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::options(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Options);
    /// ```
    pub fn options<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Options, url)
    }

    /// Create a `TRACE` request.
    ///
    /// The `TRACE` method performs a message loop-back test along the path to
    /// the target resource.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::trace(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Trace);
    /// ```
    pub fn trace<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Trace, url)
    }

    /// Create a `PATCH` request.
    ///
    /// The `PATCH` method is used to apply partial modifications to a resource.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::{Method, Request, Url};
    ///
    /// let mut req = Request::patch(Url::parse("https://example.com").unwrap());
    /// assert_eq!(req.method(), Method::Patch);
    /// ```
    pub fn patch<U>(url: U) -> Self
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        Request::new(Method::Patch, url)
    }
}

impl Clone for Request {
    /// Clone the request, resolving the body to `Body::empty()` and removing
    /// extensions.
    fn clone(&self) -> Self {
        Request {
            method: self.method.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            version: self.version.clone(),
            trailers_sender: None,
            trailers_receiver: None,
            body: Body::empty(),
            ext: Extensions::new(),
            peer_addr: self.peer_addr.clone(),
            local_addr: self.local_addr.clone(),
        }
    }
}

impl Read for Request {
    #[allow(missing_doc_code_examples)]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.body).poll_read(cx, buf)
    }
}

impl BufRead for Request {
    #[allow(missing_doc_code_examples)]
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&'_ [u8]>> {
        let this = self.project();
        this.body.poll_fill_buf(cx)
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.body).consume(amt)
    }
}

impl AsRef<Headers> for Request {
    fn as_ref(&self) -> &Headers {
        &self.headers
    }
}

impl AsMut<Headers> for Request {
    fn as_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }
}

impl From<Request> for Body {
    fn from(req: Request) -> Body {
        req.body
    }
}

impl Index<HeaderName> for Request {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Request`.
    #[inline]
    fn index(&self, name: HeaderName) -> &HeaderValues {
        self.headers.index(name)
    }
}

impl Index<&str> for Request {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Request`.
    #[inline]
    fn index(&self, name: &str) -> &HeaderValues {
        self.headers.index(name)
    }
}

impl IntoIterator for Request {
    type Item = (HeaderName, HeaderValues);
    type IntoIter = headers::IntoIter;

    /// Returns a iterator of references over the remaining items.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.headers.into_iter()
    }
}

impl<'a> IntoIterator for &'a Request {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.headers.iter()
    }
}

impl<'a> IntoIterator for &'a mut Request {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.headers.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod host {
        use super::*;

        #[test]
        fn when_forwarded_header_is_set() {
            let mut request = build_test_request();
            set_forwarded(&mut request, "-");
            set_x_forwarded_host(&mut request, "this will not be used");
            assert_eq!(request.forwarded_header_part("host"), Some("host.com"));
            assert_eq!(request.host(), Some("host.com"));
        }

        #[test]
        fn when_several_x_forwarded_hosts_exist() {
            let mut request = build_test_request();
            set_x_forwarded_host(&mut request, "expected.host");

            assert_eq!(request.forwarded_header_part("host"), None);
            assert_eq!(request.host(), Some("expected.host"));
        }

        #[test]
        fn when_only_one_x_forwarded_hosts_exist() {
            let mut request = build_test_request();
            request.insert_header("x-forwarded-host", "expected.host");
            assert_eq!(request.host(), Some("expected.host"));
        }

        #[test]
        fn when_host_header_is_set() {
            let mut request = build_test_request();
            request.insert_header("host", "host.header");
            assert_eq!(request.host(), Some("host.header"));
        }

        #[test]
        fn when_there_are_no_headers() {
            let request = build_test_request();
            assert_eq!(request.host(), Some("async.rs"));
        }

        #[test]
        fn when_url_has_no_domain() {
            let mut request = build_test_request();
            *request.url_mut() = Url::parse("x:").unwrap();
            assert_eq!(request.host(), None);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_get() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::get(url);
            assert_eq!(req.method(), Method::Get);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_head() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::head(url);
            assert_eq!(req.method(), Method::Head);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_post() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::post(url);
            assert_eq!(req.method(), Method::Post);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_put() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::put(url);
            assert_eq!(req.method(), Method::Put);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_delete() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::delete(url);
            assert_eq!(req.method(), Method::Delete);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_connect() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::connect(url);
            assert_eq!(req.method(), Method::Connect);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_options() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::options(url);
            assert_eq!(req.method(), Method::Options);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_trace() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::trace(url);
            assert_eq!(req.method(), Method::Trace);
        }

        #[test]
        fn when_using_shorthand_with_valid_url_to_create_request_patch() {
            let url = Url::parse("https://example.com").unwrap();
            let req = Request::patch(url);
            assert_eq!(req.method(), Method::Patch);
        }
    }

    mod remote {
        use super::*;
        #[test]
        fn when_forwarded_is_properly_formatted() {
            let mut request = build_test_request();
            request.set_peer_addr(Some("127.0.0.1:8000"));
            set_forwarded(&mut request, "127.0.0.1:8001");

            assert_eq!(request.forwarded_for(), Some("127.0.0.1:8001"));
            assert_eq!(request.remote(), Some("127.0.0.1:8001"));
        }

        #[test]
        fn when_forwarded_is_improperly_formatted() {
            let mut request = build_test_request();
            request.set_peer_addr(Some(
                "127.0.0.1:8000".parse::<std::net::SocketAddr>().unwrap(),
            ));

            request.insert_header("Forwarded", "this is an improperly ;;; formatted header");

            assert_eq!(request.forwarded_for(), None);
            assert_eq!(request.remote(), Some("127.0.0.1:8000"));
        }

        #[test]
        fn when_x_forwarded_for_is_set() {
            let mut request = build_test_request();
            request.set_peer_addr(Some(
                std::path::PathBuf::from("/dev/random").to_str().unwrap(),
            ));
            set_x_forwarded_for(&mut request, "forwarded-host.com");

            assert_eq!(request.forwarded_for(), Some("forwarded-host.com"));
            assert_eq!(request.remote(), Some("forwarded-host.com"));
        }

        #[test]
        fn when_both_forwarding_headers_are_set() {
            let mut request = build_test_request();
            set_forwarded(&mut request, "forwarded.com");
            set_x_forwarded_for(&mut request, "forwarded-for-client.com");
            request.peer_addr = Some("127.0.0.1:8000".into());

            assert_eq!(request.forwarded_for(), Some("forwarded.com".into()));
            assert_eq!(request.remote(), Some("forwarded.com".into()));
        }

        #[test]
        fn falling_back_to_peer_addr() {
            let mut request = build_test_request();
            request.peer_addr = Some("127.0.0.1:8000".into());

            assert_eq!(request.forwarded_for(), None);
            assert_eq!(request.remote(), Some("127.0.0.1:8000".into()));
        }

        #[test]
        fn when_no_remote_available() {
            let request = build_test_request();
            assert_eq!(request.forwarded_for(), None);
            assert_eq!(request.remote(), None);
        }
    }

    fn build_test_request() -> Request {
        let url = Url::parse("http://async.rs/").unwrap();
        Request::new(Method::Get, url)
    }

    fn set_x_forwarded_for(request: &mut Request, client: &'static str) {
        request.insert_header(
            "x-forwarded-for",
            format!("{},proxy.com,other-proxy.com", client),
        );
    }

    fn set_x_forwarded_host(request: &mut Request, host: &'static str) {
        request.insert_header(
            "x-forwarded-host",
            format!("{},proxy.com,other-proxy.com", host),
        );
    }

    fn set_forwarded(request: &mut Request, client: &'static str) {
        request.insert_header(
            "Forwarded",
            format!("by=something.com;for={};host=host.com;proto=http", client),
        );
    }
}
