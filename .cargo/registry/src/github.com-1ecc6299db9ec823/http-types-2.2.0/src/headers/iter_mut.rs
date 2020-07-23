use std::collections::hash_map;
use std::iter::Iterator;

use crate::headers::{HeaderName, HeaderValues};

/// Iterator over the headers.
#[derive(Debug)]
pub struct IterMut<'a> {
    pub(super) inner: hash_map::IterMut<'a, HeaderName, HeaderValues>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a HeaderName, &'a mut HeaderValues);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
