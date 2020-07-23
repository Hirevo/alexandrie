# infer

![Build Status](https://github.com/bojand/infer/workflows/build/badge.svg)
[![crates version](https://img.shields.io/crates/v/infer.svg)](https://crates.io/crates/infer)
[![documentation](https://docs.rs/infer/badge.svg)](https://docs.rs/infer)

Small crate to infer file and MIME type by checking the
[magic number](https://en.wikipedia.org/wiki/Magic_number_(programming)) signature. 

Adaptation of [filetype](https://github.com/h2non/filetype) Go package ported to Rust. 

Does not require magic file database (i.e. `/etc/magic`). 

## Features

- Supports a [wide range](#supported-types) of file types
- Provides file extension and MIME type
- File discovery by extension or MIME type
- File discovery by class (image, video, audio...)
- Supports custom new types and matchers

## Documentation

https://docs.rs/infer

## Installation

This crate works with Cargo and is on
[crates.io](https://crates.io/crates/infer). Add it to your `Cargo.toml`
like so:

```toml
[dependencies]
infer = "0.1"
```

## Examples

### Get the type of a buffer

```rust
use infer::Infer;

let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
let info = Infer::new();

assert_eq!("image/jpeg", info.get(&v).unwrap().mime);
assert_eq!("jpg", info.get(&v).unwrap().ext);
```

### Check path

```rust
use infer::Infer;

let info = Infer::new();
let res = info.get_from_path("testdata/sample.jpg");

assert!(res.is_ok());
let o = res.unwrap();
assert!(o.is_some());
let typ = o.unwrap();

assert_eq!("image/jpeg", typ.mime);
assert_eq!("jpg", typ.ext);
```

### Check for specific type

```rust
let v = vec![0xFF, 0xD8, 0xFF, 0xAA];
assert!(infer::image::is_jpeg(&v));
```

### Check for specific type class
Note individual matcher functions do not require init

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

## Supported types

#### Image

- **jpg** - `image/jpeg`
- **png** - `image/png`
- **gif** - `image/gif`
- **webp** - `image/webp`
- **cr2** - `image/x-canon-cr2`
- **tif** - `image/tiff`
- **bmp** - `image/bmp`
- **heif** - `image/heif`
- **jxr** - `image/vnd.ms-photo`
- **psd** - `image/vnd.adobe.photoshop`
- **ico** - `image/x-icon`

#### Video

- **mp4** - `video/mp4`
- **m4v** - `video/x-m4v`
- **mkv** - `video/x-matroska`
- **webm** - `video/webm`
- **mov** - `video/quicktime`
- **avi** - `video/x-msvideo`
- **wmv** - `video/x-ms-wmv`
- **mpg** - `video/mpeg`
- **flv** - `video/x-flv`

#### Audio

- **mid** - `audio/midi`
- **mp3** - `audio/mpeg`
- **m4a** - `audio/m4a`
- **ogg** - `audio/ogg`
- **flac** - `audio/x-flac`
- **wav** - `audio/x-wav`
- **amr** - `audio/amr`
- **aac** - `audio/aac`

#### Archive

- **epub** - `application/epub+zip`
- **zip** - `application/zip`
- **tar** - `application/x-tar`
- **rar** - `application/x-rar-compressed`
- **gz** - `application/gzip`
- **bz2** - `application/x-bzip2`
- **7z** - `application/x-7z-compressed`
- **xz** - `application/x-xz`
- **pdf** - `application/pdf`
- **swf** - `application/x-shockwave-flash`
- **rtf** - `application/rtf`
- **eot** - `application/octet-stream`
- **ps** - `application/postscript`
- **sqlite** - `application/x-sqlite3`
- **nes** - `application/x-nintendo-nes-rom`
- **crx** - `application/x-google-chrome-extension`
- **cab** - `application/vnd.ms-cab-compressed`
- **deb** - `application/x-deb`
- **ar** - `application/x-unix-archive`
- **Z** - `application/x-compress`
- **lz** - `application/x-lzip`
- **rpm** - `application/x-rpm`
- **dcm** - `application/dicom`
- **zst** - `application/zstd`

#### Documents

- **doc** - `application/msword`
- **docx** - `application/vnd.openxmlformats-officedocument.wordprocessingml.document`
- **xls** - `application/vnd.ms-excel`
- **xlsx** - `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`
- **ppt** - `application/vnd.ms-powerpoint`
- **pptx** - `application/vnd.openxmlformats-officedocument.presentationml.presentation`

#### Font

- **woff** - `application/font-woff`
- **woff2** - `application/font-woff`
- **ttf** - `application/font-sfnt`
- **otf** - `application/font-sfnt`

#### Application

- **wasm** - `application/wasm`
- **exe** - `application/x-msdownload`
- **elf** - `application/x-executable`
- **bc** - `application/llvm`
- **class** - `application/java`

## Known Issues

- `doc`, `ppt`, `xls` all have the same magic number so it's not possible to tell which one just based on the binary data. `doc` is returned for all.

## License

MIT