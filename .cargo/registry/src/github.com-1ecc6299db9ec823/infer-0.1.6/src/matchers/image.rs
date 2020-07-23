extern crate byteorder;

use byteorder::{BigEndian, ByteOrder};
use std::str;

/// Returns whether a buffer is JPEG image data.
pub fn is_jpeg(buf: &[u8]) -> bool {
    buf.len() > 2 && buf[0] == 0xFF && buf[1] == 0xD8 && buf[2] == 0xFF
}

/// Returns whether a buffer is jpg2 image data.
pub fn is_jpeg2000(buf: &[u8]) -> bool {
    buf.len() > 12
        && buf[0] == 0x0
        && buf[1] == 0x0
        && buf[2] == 0x0
        && buf[3] == 0xC
        && buf[4] == 0x6A
        && buf[5] == 0x50
        && buf[6] == 0x20
        && buf[7] == 0x20
        && buf[8] == 0xD
        && buf[9] == 0xA
        && buf[10] == 0x87
        && buf[11] == 0xA
        && buf[12] == 0x0
}

/// Returns whether a buffer is PNG image data.
pub fn is_png(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x89 && buf[1] == 0x50 && buf[2] == 0x4E && buf[3] == 0x47
}

/// Returns whether a buffer is GIF image data.
pub fn is_gif(buf: &[u8]) -> bool {
    buf.len() > 2 && buf[0] == 0x47 && buf[1] == 0x49 && buf[2] == 0x46
}

/// Returns whether a buffer is WEBP image data.
pub fn is_webp(buf: &[u8]) -> bool {
    buf.len() > 11 && buf[8] == 0x57 && buf[9] == 0x45 && buf[10] == 0x42 && buf[11] == 0x50
}

/// Returns whether a buffer is Canon CR2 image data.
pub fn is_cr2(buf: &[u8]) -> bool {
    buf.len() > 9
        && ((buf[0] == 0x49 && buf[1] == 0x49 && buf[2] == 0x2A && buf[3] == 0x0)
            || (buf[0] == 0x4D && buf[1] == 0x4D && buf[2] == 0x0 && buf[3] == 0x2A))
        && buf[8] == 0x43
        && buf[9] == 0x52
}

/// Returns whether a buffer is TIFF image data.
pub fn is_tiff(buf: &[u8]) -> bool {
    buf.len() > 9
        && ((buf[0] == 0x49 && buf[1] == 0x49 && buf[2] == 0x2A && buf[3] == 0x0)
            || (buf[0] == 0x4D && buf[1] == 0x4D && buf[2] == 0x0 && buf[3] == 0x2A))
        && buf[8] != 0x43
        && buf[9] != 0x52
}

/// Returns whether a buffer is BMP image data.
pub fn is_bmp(buf: &[u8]) -> bool {
    buf.len() > 1 && buf[0] == 0x42 && buf[1] == 0x4D
}

/// Returns whether a buffer is jxr image data.
pub fn is_jxr(buf: &[u8]) -> bool {
    buf.len() > 2 && buf[0] == 0x49 && buf[1] == 0x49 && buf[2] == 0xBC
}

/// Returns whether a buffer is Photoshop PSD image data.
pub fn is_psd(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x38 && buf[1] == 0x42 && buf[2] == 0x50 && buf[3] == 0x53
}

/// Returns whether a buffer is ICO icon image data.
pub fn is_ico(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x00 && buf[1] == 0x00 && buf[2] == 0x01 && buf[3] == 0x00
}

/// Returns whether a buffer is HEIF image data.
pub fn is_heif(buf: &[u8]) -> bool {
    if buf.is_empty() {
        return false;
    }

    if !is_isobmff(buf) {
        return false;
    }

    match get_ftyp(buf) {
        None => false,
        Some((major, _minor, compatible)) => {
            if major == "heic" {
                return true;
            }

            if major == "mif1" || major == "msf1" {
                for b in compatible {
                    if b == "heic" {
                        return true;
                    }
                }
            }

            false
        }
    }
}

// IsISOBMFF checks whether the given buffer represents ISO Base Media File Format data
fn is_isobmff(buf: &[u8]) -> bool {
    if buf.len() < 8 {
        return false;
    }

    let slice_str = str::from_utf8(&buf[4..8]);
    if slice_str.is_err() {
        return false;
    }

    if buf.len() < 16 || slice_str.unwrap() != "ftyp" {
        return false;
    }

    let ftyp_length = BigEndian::read_u32(&buf[0..4]) as usize;
    buf.len() >= ftyp_length
}

// GetFtyp returns the major brand, minor version and compatible brands of the ISO-BMFF data
fn get_ftyp(buf: &[u8]) -> Option<(String, String, Vec<String>)> {
    if buf.len() < 16 {
        return None;
    }

    let ftyp_length = BigEndian::read_u32(&buf[0..4]) as usize;

    let major_str = str::from_utf8(&buf[8..12]);
    let minor_str = str::from_utf8(&buf[12..16]);

    if major_str.is_err() || minor_str.is_err() {
        return None;
    }

    let major = String::from(major_str.unwrap());
    let minor = String::from(minor_str.unwrap());

    let mut compatible = Vec::new();
    for i in (16..ftyp_length).step_by(4) {
        let v = str::from_utf8(&buf[i..i + 4]).unwrap();
        compatible.push(String::from(v));
    }

    Some((major, minor, compatible))
}
