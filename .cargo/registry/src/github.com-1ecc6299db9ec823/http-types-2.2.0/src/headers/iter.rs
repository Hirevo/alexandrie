use std::collections::hash_map;
use std::iter::Iterator;

use crate::headers::{HeaderName, HeaderValues};

/// Iterator over the headers.
#[derive(Debug)]
pub struct Iter<'a> {
    pub(super) inner: hash_map::Iter<'a, HeaderName, HeaderValues>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a HeaderName, &'a HeaderValues);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
