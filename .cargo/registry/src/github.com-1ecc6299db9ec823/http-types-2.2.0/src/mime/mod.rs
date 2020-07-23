//! IANA Media Types.
//!
//! [Read more](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types).

mod constants;
mod parse;

pub use constants::*;

use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::option;
use std::str::FromStr;

use crate::headers::{HeaderValue, ToHeaderValues};

use infer::Infer;

/// An IANA media type.
///
/// ```
/// use http_types::Mime;
/// use std::str::FromStr;
///
/// let mime = Mime::from_str("text/html;charset=utf-8").unwrap();
/// assert_eq!(mime.essence(), "text/html");
/// assert_eq!(mime.param("charset").unwrap(), "utf-8");
/// ```
// NOTE: we cannot statically initialize Strings with values yet, so we keep dedicated static
// fields for the static strings.
#[derive(Clone)]
pub struct Mime {
    pub(crate) essence: String,
    pub(crate) basetype: String,
    pub(crate) subtype: String,
    pub(crate) static_essence: Option<&'static str>,
    pub(crate) static_basetype: Option<&'static str>,
    pub(crate) static_subtype: Option<&'static str>,
    pub(crate) params: Option<ParamKind>,
}

impl Mime {
    /// Sniff the mime type from a byte slice.
    pub fn sniff(bytes: &[u8]) -> crate::Result<Self> {
        let info = Infer::new();
        let mime = match info.get(&bytes) {
            Some(info) => info.mime,
            None => crate::bail!("Could not sniff the mime type"),
        };
        Mime::from_str(&mime)
    }

    /// Access the Mime's `type` value.
    ///
    /// According to the spec this method should be named `type`, but that's a reserved keyword in
    /// Rust so hence prefix with `base` instead.
    pub fn basetype(&self) -> &str {
        if let Some(basetype) = self.static_basetype {
            &basetype
        } else {
            &self.basetype
        }
    }

    /// Access the Mime's `subtype` value.
    pub fn subtype(&self) -> &str {
        if let Some(subtype) = self.static_subtype {
            &subtype
        } else {
            &self.subtype
        }
    }

    /// Access the Mime's `essence` value.
    pub fn essence(&self) -> &str {
        if let Some(essence) = self.static_essence {
            &essence
        } else {
            &self.essence
        }
    }

    /// Get a reference to a param.
    pub fn param(&self, name: impl Into<ParamName>) -> Option<&ParamValue> {
        let name: ParamName = name.into();
        self.params
            .as_ref()
            .map(|inner| match inner {
                ParamKind::Vec(v) => v
                    .iter()
                    .find_map(|(k, v)| if k == &name { Some(v) } else { None }),
                ParamKind::Utf8 => match name {
                    ParamName(Cow::Borrowed("charset")) => Some(&ParamValue(Cow::Borrowed("utf8"))),
                    _ => None,
                },
            })
            .flatten()
    }
}

impl PartialEq<Mime> for Mime {
    fn eq(&self, other: &Mime) -> bool {
        let left = match self.static_essence {
            Some(essence) => essence,
            None => &self.essence,
        };
        let right = match other.static_essence {
            Some(essence) => essence,
            None => &other.essence,
        };
        left == right
    }
}

impl Display for Mime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        parse::format(self, f)
    }
}

impl Debug for Mime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(essence) = self.static_essence {
            Debug::fmt(essence, f)
        } else {
            Debug::fmt(&self.essence, f)
        }
    }
}

impl FromStr for Mime {
    type Err = crate::Error;

    /// Create a new `Mime`.
    ///
    /// Follows the [WHATWG MIME parsing algorithm](https://mimesniff.spec.whatwg.org/#parsing-a-mime-type).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::parse(s)
    }
}

impl ToHeaderValues for Mime {
    type Iter = option::IntoIter<HeaderValue>;

    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        let mime = self.clone();
        let header: HeaderValue = mime.into();

        // A HeaderValue will always convert into itself.
        Ok(header.to_header_values().unwrap())
    }
}
/// A parameter name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamName(Cow<'static, str>);

impl Display for ParamName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for ParamName {
    type Err = crate::Error;

    /// Create a new `HeaderName`.
    ///
    /// This checks it's valid ASCII, and lowercases it.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::ensure!(s.is_ascii(), "String slice should be valid ASCII");
        Ok(ParamName(Cow::Owned(s.to_ascii_lowercase())))
    }
}

impl<'a> From<&'a str> for ParamName {
    fn from(value: &'a str) -> Self {
        Self::from_str(value).unwrap()
    }
}

/// A parameter value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamValue(Cow<'static, str>);

impl Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<'a> PartialEq<&'a str> for ParamValue {
    fn eq(&self, other: &&'a str) -> bool {
        &self.0 == other
    }
}

impl PartialEq<str> for ParamValue {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

/// This is a hack that allows us to mark a trait as utf8 during compilation. We
/// can remove this once we can construct HashMap during compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParamKind {
    Utf8,
    Vec(Vec<(ParamName, ParamValue)>),
}
