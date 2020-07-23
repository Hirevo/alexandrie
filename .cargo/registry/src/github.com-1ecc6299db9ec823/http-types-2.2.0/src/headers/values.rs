use std::collections::hash_map;
use std::iter::Iterator;

use crate::headers::{HeaderName, HeaderValue, HeaderValues};

/// Iterator over the header values.
#[derive(Debug)]
pub struct Values<'a> {
    pub(super) inner: Option<hash_map::Values<'a, HeaderName, HeaderValues>>,
    slot: Option<&'a HeaderValues>,
    cursor: usize,
}

impl<'a> Values<'a> {
    /// Constructor for `Headers`.
    pub(crate) fn new(inner: hash_map::Values<'a, HeaderName, HeaderValues>) -> Self {
        Self {
            inner: Some(inner),
            slot: None,
            cursor: 0,
        }
    }

    /// Constructor for `HeaderValues`.
    pub(crate) fn new_values(values: &'a HeaderValues) -> Self {
        Self {
            inner: None,
            slot: Some(values),
            cursor: 0,
        }
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Check if we have a vec in the current slot, and if not set one.
            if self.slot.is_none() {
                let next = match self.inner.as_mut() {
                    Some(inner) => inner.next()?,
                    None => return None,
                };
                self.cursor = 0;
                self.slot = Some(next);
            }

            // Get the next item
            match self.slot.unwrap().get(self.cursor) {
                // If an item is found, increment the cursor and return the item.
                Some(item) => {
                    self.cursor += 1;
                    return Some(item);
                }
                // If no item is found, unset the slot and loop again.
                None => {
                    self.slot = None;
                    continue;
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.inner.as_ref() {
            Some(inner) => inner.size_hint(),
            None => (0, None),
        }
    }
}
