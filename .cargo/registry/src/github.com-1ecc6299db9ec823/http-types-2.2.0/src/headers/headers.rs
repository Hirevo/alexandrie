//! HTTP headers.

use std::collections::HashMap;
use std::convert::Into;
use std::fmt::{self, Debug};
use std::iter::IntoIterator;
use std::ops::Index;
use std::str::FromStr;

use crate::headers::{
    HeaderName, HeaderValues, IntoIter, Iter, IterMut, Names, ToHeaderValues, Values,
};

/// A collection of HTTP Headers.
///
/// Headers are never manually constructed, but are part of `Request`,
/// `Response`, and `Trailers`. Each of these types implements `AsRef<Headers>`
/// and `AsMut<Headers>` so functions that want to modify headers can be generic
/// over either of these traits.
///
/// # Examples
///
/// ```
/// use http_types::{Response, StatusCode};
///
/// let mut res = Response::new(StatusCode::Ok);
/// res.insert_header("hello", "foo0");
/// assert_eq!(res["hello"], "foo0");
/// ```
#[derive(Clone)]
pub struct Headers {
    pub(crate) headers: HashMap<HeaderName, HeaderValues>,
}

impl Headers {
    /// Create a new instance.
    pub(crate) fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// Insert a header into the headers.
    ///
    /// Not that this will replace all header values for a given header name.
    /// If you wish to add header values for a header name that already exists
    /// use `Headers::append`
    pub fn insert(
        &mut self,
        name: impl Into<HeaderName>,
        values: impl ToHeaderValues,
    ) -> Option<HeaderValues> {
        let name = name.into();
        let values: HeaderValues = values.to_header_values().unwrap().collect();
        self.headers.insert(name, values)
    }

    /// Append a header to the headers.
    ///
    /// Unlike `insert` this function will not override the contents of a header, but insert a
    /// header if there aren't any. Or else append to the existing list of headers.
    pub fn append(&mut self, name: impl Into<HeaderName>, values: impl ToHeaderValues) {
        let name = name.into();
        match self.get_mut(name.clone()) {
            Some(headers) => {
                let mut values: HeaderValues = values.to_header_values().unwrap().collect();
                headers.append(&mut values);
            }
            None => {
                self.insert(name, values);
            }
        }
    }

    /// Get a reference to a header.
    pub fn get(&self, name: impl Into<HeaderName>) -> Option<&HeaderValues> {
        self.headers.get(&name.into())
    }

    /// Get a mutable reference to a header.
    pub fn get_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValues> {
        self.headers.get_mut(&name.into())
    }

    /// Remove a header.
    pub fn remove(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.headers.remove(&name.into())
    }

    /// An iterator visiting all header pairs in arbitrary order.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.headers.iter(),
        }
    }

    /// An iterator visiting all header pairs in arbitrary order, with mutable references to the
    /// values.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.headers.iter_mut(),
        }
    }

    /// An iterator visiting all header names in arbitrary order.
    pub fn names(&self) -> Names<'_> {
        Names {
            inner: self.headers.keys(),
        }
    }

    /// An iterator visiting all header values in arbitrary order.
    pub fn values(&self) -> Values<'_> {
        Values::new(self.headers.values())
    }
}

impl Index<HeaderName> for Headers {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Headers`.
    #[inline]
    fn index(&self, name: HeaderName) -> &HeaderValues {
        self.get(name).expect("no entry found for name")
    }
}

impl Index<&str> for Headers {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Headers`.
    #[inline]
    fn index(&self, name: &str) -> &HeaderValues {
        let name = HeaderName::from_str(name).expect("string slice needs to be valid ASCII");
        self.get(name).expect("no entry found for name")
    }
}

impl IntoIterator for Headers {
    type Item = (HeaderName, HeaderValues);
    type IntoIter = IntoIter;

    /// Returns a iterator of references over the remaining items.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.headers.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Headers {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Debug for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.headers.iter()).finish()
    }
}

impl AsRef<Headers> for Headers {
    fn as_ref(&self) -> &Headers {
        self
    }
}

impl AsMut<Headers> for Headers {
    fn as_mut(&mut self) -> &mut Headers {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const STATIC_HEADER: HeaderName = HeaderName::from_lowercase_str("hello");

    #[test]
    fn test_header_name_static_non_static() -> crate::Result<()> {
        let static_header = HeaderName::from_lowercase_str("hello");
        let non_static_header = HeaderName::from_str("hello")?;

        let mut headers = Headers::new();
        headers.append(STATIC_HEADER, "foo0");
        headers.append(static_header.clone(), "foo1");
        headers.append(non_static_header.clone(), "foo2");

        assert_eq!(headers[STATIC_HEADER], ["foo0", "foo1", "foo2",][..]);
        assert_eq!(headers[static_header], ["foo0", "foo1", "foo2",][..]);
        assert_eq!(headers[non_static_header], ["foo0", "foo1", "foo2",][..]);

        Ok(())
    }

    #[test]
    fn index_into_headers() {
        let mut headers = Headers::new();
        headers.insert("hello", "foo0");
        assert_eq!(headers["hello"], "foo0");
        assert_eq!(headers.get("hello").unwrap(), "foo0");
    }

    #[test]
    fn test_debug_single() {
        let mut headers = Headers::new();
        headers.insert("single", "foo0");
        assert_eq!(format!("{:?}", headers), r#"{"single": "foo0"}"#);
    }

    #[test]
    fn test_debug_multiple() {
        let mut headers = Headers::new();
        headers.append("multi", "foo0");
        headers.append("multi", "foo1");
        assert_eq!(format!("{:?}", headers), r#"{"multi": ["foo0", "foo1"]}"#);
    }
}
