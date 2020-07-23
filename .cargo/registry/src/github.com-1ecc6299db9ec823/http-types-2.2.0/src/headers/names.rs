use std::collections::hash_map;
use std::iter::Iterator;

use crate::headers::{HeaderName, HeaderValues};

/// Iterator over the headers.
#[derive(Debug)]
pub struct Names<'a> {
    pub(super) inner: hash_map::Keys<'a, HeaderName, HeaderValues>,
}

impl<'a> Iterator for Names<'a> {
    type Item = &'a HeaderName;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
