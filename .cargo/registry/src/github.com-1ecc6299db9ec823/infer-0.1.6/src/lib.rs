/*!
Small crate to infer file and MIME type by checking the
[magic number](https://en.wikipedia.org/wiki/Magic_number_(programming)) signature.

# Examples

### Get the type of a buffer

```rust
let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
let info = infer::Infer::new();
assert_eq!("image/jpeg", info.get(&v).unwrap().mime);
assert_eq!("jpg", info.get(&v).unwrap().ext);
```

### Check path

```rust
let info = infer::Infer::new();
let res = info.get_from_path("testdata/sample.jpg");
assert!(res.is_ok());
let o = res.unwrap();
assert!(o.is_some());
let typ = o.unwrap();
assert_eq!("image/jpeg", typ.mime);
assert_eq!("jpg", typ.ext);
```

### Check for specific type

Note individual matcher functions do not require an Infer struct instance.

```rust
let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
assert!(infer::image::is_jpeg(&v));
```

### Check for specific type class

```rust
let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
let info = infer::Infer::new();
assert!(info.is_image(&v));
```

### Adds a custom file type matcher

```rust
fn custom_matcher(buf: &[u8]) -> bool {
    return buf.len() >= 3 && buf[0] == 0x10 && buf[1] == 0x11 && buf[2] == 0x12;
}

let mut info = infer::Infer::new();
info.add("custom/foo", "foo", custom_matcher);

let v = vec![0x10, 0x11, 0x12, 0x13];
let res =  info.get(&v).unwrap();

assert_eq!("custom/foo", res.mime);
assert_eq!("foo", res.ext);
```
*/
#![crate_name = "infer"]

mod map;
mod matchers;

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// All the supported matchers categorized and exposed as functions
pub use matchers::*;

/// Matcher function
pub type Matcher = fn(buf: &[u8]) -> bool;

/// Generic information for a type
#[derive(Debug, Eq, PartialEq)]
pub struct Type {
    /// The mime
    pub mime: String,

    /// The file extension
    pub ext: String,
}

/// Infer is the main struct of the module
pub struct Infer {
    mmap: Vec<(map::MatcherType, String, String, Matcher)>,
}

impl Infer {
    /// Initialize a new instance of the infer struct.
    pub fn new() -> Infer {
        let mut v: Vec<(map::MatcherType, String, String, Matcher)> = Vec::new();

        map::setup(&mut v);

        Infer { mmap: v }
    }

    /// Returns the file type of the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let info = infer::Infer::new();
    /// let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
    /// assert_eq!("image/jpeg", info.get(&v).unwrap().mime);
    /// assert_eq!("jpg", info.get(&v).unwrap().ext);
    /// ```
    pub fn get(&self, buf: &[u8]) -> Option<Type> {
        for (_mt, mime, ext, matcher) in self.mmap.iter() {
            if matcher(buf) {
                return Some(Type {
                    mime: (*mime).clone(),
                    ext: (*ext).clone(),
                });
            }
        }

        None
    }

    /// Returns the file type of the file given a path.
    ///
    /// # Errors
    ///
    /// Returns an error if we fail to read the path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let info = infer::Infer::new();
    /// let res = info.get_from_path("testdata/sample.jpg");
    /// assert!(res.is_ok());
    /// let o = res.unwrap();
    /// assert!(o.is_some());
    /// let typ = o.unwrap();
    /// assert_eq!("image/jpeg", typ.mime);
    /// assert_eq!("jpg", typ.ext);
    /// ```
    pub fn get_from_path<P: AsRef<Path>>(&self, path: P) -> Result<Option<Type>, std::io::Error> {
        let file = File::open(path)?;

        let limit = file
            .metadata()
            .map(|m| std::cmp::min(m.len(), 8192) as usize + 1)
            .unwrap_or(0);
        let mut bytes = Vec::with_capacity(limit);
        file.take(8192).read_to_end(&mut bytes)?;

        Ok(self.get(&bytes))
    }

    /// Determines whether a buffer is of given extension.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let info = infer::Infer::new();
    /// let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
    /// assert!(info.is(&v, "jpg"));
    /// ```
    pub fn is(&self, buf: &[u8], ext: &str) -> bool {
        if let Some((_mt, _mi, _e, matcher)) = self
            .mmap
            .iter()
            .find(|(_mt, _mime, ex, _matcher)| *ex == ext)
        {
            if matcher(buf) {
                return true;
            }
        }

        false
    }

    /// Determines whether a buffer is of given mime type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let info = infer::Infer::new();
    /// let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
    /// assert!(info.is_mime(&v, "image/jpeg"));
    /// ```
    pub fn is_mime(&self, buf: &[u8], mime: &str) -> bool {
        if let Some((_mt, _mi, _e, matcher)) = self
            .mmap
            .iter()
            .find(|(_mt, mi, _ext, _matcher)| *mi == mime)
        {
            if matcher(buf) {
                return true;
            }
        }

        false
    }

    /// Returns whether an extension is supported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let info = infer::Infer::new();
    /// assert!(info.is_supported("jpg"));
    /// ```
    pub fn is_supported(&self, ext: &str) -> bool {
        for (_mt, _mime, type_ext, _matcher) in self.mmap.iter() {
            if ext == *type_ext {
                return true;
            }
        }

        false
    }

    /// Returns whether a mime type is supported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let info = infer::Infer::new();
    /// assert!(info.is_mime_supported("image/jpeg"));
    /// ```
    pub fn is_mime_supported(&self, mime: &str) -> bool {
        for (_mt, type_mime, _ext, _matcher) in self.mmap.iter() {
            if mime == *type_mime {
                return true;
            }
        }

        false
    }

    /// Determines whether a buffer is an application type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::fs;
    /// let info = infer::Infer::new();
    /// assert!(info.is_app(&fs::read("testdata/sample.wasm").unwrap()));
    /// ```
    pub fn is_app(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::APP)
    }

    /// Determines whether a buffer is an archive type.
    /// # Examples
    ///
    /// ```rust
    /// use std::fs;
    /// let info = infer::Infer::new();
    /// assert!(info.is_archive(&fs::read("testdata/sample.pdf").unwrap()));
    /// ```
    pub fn is_archive(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::ARCHIVE)
    }

    /// Determines whether a buffer is an audio type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // mp3
    /// let info = infer::Infer::new();
    /// let v = vec![0xff, 0xfb, 0x90, 0x44, 0x00];
    /// assert!(info.is_audio(&v));
    /// ```
    pub fn is_audio(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::AUDIO)
    }

    /// Determines whether a buffer is a document type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::fs;
    /// let info = infer::Infer::new();
    /// assert!(info.is_document(&fs::read("testdata/sample.docx").unwrap()));
    /// ```
    pub fn is_document(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::DOC)
    }

    /// Determines whether a buffer is a font type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::fs;
    /// let info = infer::Infer::new();
    /// assert!(info.is_font(&fs::read("testdata/sample.ttf").unwrap()));
    /// ```
    pub fn is_font(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::FONT)
    }

    /// Determines whether a buffer is an image type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
    /// let info = infer::Infer::new();
    /// assert!(info.is_image(&v));
    /// ```
    pub fn is_image(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::IMAGE)
    }

    /// Determines whether a buffer is a video type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::fs;
    /// let info = infer::Infer::new();
    /// assert!(info.is_video(&fs::read("testdata/sample.mov").unwrap()));
    /// ```
    pub fn is_video(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::VIDEO)
    }

    /// Determines whether a buffer is one of the custom types added.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn custom_matcher(buf: &[u8]) -> bool {
    ///     return buf.len() >= 3 && buf[0] == 0x10 && buf[1] == 0x11 && buf[2] == 0x12;
    /// }
    ///
    /// let mut info = infer::Infer::new();
    /// info.add("custom/foo", "foo", custom_matcher);
    /// let v = vec![0x10, 0x11, 0x12, 0x13];
    /// assert!(info.is_custom(&v));
    /// ```
    pub fn is_custom(&self, buf: &[u8]) -> bool {
        self.is_type(buf, map::MatcherType::CUSTOM)
    }

    /// Adds a custom matcher.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn custom_matcher(buf: &[u8]) -> bool {
    ///     return buf.len() >= 3 && buf[0] == 0x10 && buf[1] == 0x11 && buf[2] == 0x12;
    /// }
    ///
    /// let mut info = infer::Infer::new();
    /// info.add("custom/foo", "foo", custom_matcher);
    /// let v = vec![0x10, 0x11, 0x12, 0x13];
    /// let res =  info.get(&v).unwrap();
    /// assert_eq!("custom/foo", res.mime);
    /// assert_eq!("foo", res.ext);
    /// ```
    pub fn add(&mut self, mime: &str, ext: &str, m: Matcher) {
        self.mmap.push((
            map::MatcherType::CUSTOM,
            mime.to_string(),
            ext.to_string(),
            m,
        ));
    }

    fn is_type(&self, buf: &[u8], typ: map::MatcherType) -> bool {
        for (_mt, _mi, _ex, matcher) in self
            .mmap
            .iter()
            .filter(|(mt, _mime, _e, _matcher)| *mt == typ)
        {
            if matcher(buf) {
                return true;
            }
        }

        false
    }
}

impl Default for Infer {
    fn default() -> Self {
        Infer::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Infer;

    #[test]
    fn test_get_unknown() {
        let v = Vec::new();
        let info = Infer::new();
        assert!(info.get(&v).is_none());
    }

    #[test]
    fn test_get_jpeg() {
        let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
        let info = Infer::new();
        match info.get(&v) {
            Some(info) => {
                assert_eq!(info.ext, "jpg");
                assert_eq!(info.mime, "image/jpeg");
            }
            None => panic!("type info expected"),
        }
    }
}
