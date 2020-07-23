/// Returns whether a buffer is an ePub.
pub fn is_epub(buf: &[u8]) -> bool {
    buf.len() > 57
        && buf[0] == 0x50
        && buf[1] == 0x4B
        && buf[2] == 0x3
        && buf[3] == 0x4
        && buf[30] == 0x6D
        && buf[31] == 0x69
        && buf[32] == 0x6D
        && buf[33] == 0x65
        && buf[34] == 0x74
        && buf[35] == 0x79
        && buf[36] == 0x70
        && buf[37] == 0x65
        && buf[38] == 0x61
        && buf[39] == 0x70
        && buf[40] == 0x70
        && buf[41] == 0x6C
        && buf[42] == 0x69
        && buf[43] == 0x63
        && buf[44] == 0x61
        && buf[45] == 0x74
        && buf[46] == 0x69
        && buf[47] == 0x6F
        && buf[48] == 0x6E
        && buf[49] == 0x2F
        && buf[50] == 0x65
        && buf[51] == 0x70
        && buf[52] == 0x75
        && buf[53] == 0x62
        && buf[54] == 0x2B
        && buf[55] == 0x7A
        && buf[56] == 0x69
        && buf[57] == 0x70
}

/// Returns whether a buffer is a zip archive.
pub fn is_zip(buf: &[u8]) -> bool {
    buf.len() > 3
        && buf[0] == 0x50
        && buf[1] == 0x4B
        && (buf[2] == 0x3 || buf[2] == 0x5 || buf[2] == 0x7)
        && (buf[3] == 0x4 || buf[3] == 0x6 || buf[3] == 0x8)
}

/// Returns whether a buffer is a tar archive.
pub fn is_tar(buf: &[u8]) -> bool {
    buf.len() > 261
        && buf[257] == 0x75
        && buf[258] == 0x73
        && buf[259] == 0x74
        && buf[260] == 0x61
        && buf[261] == 0x72
}

/// Returns whether a buffer is a RAR archive.
pub fn is_rar(buf: &[u8]) -> bool {
    buf.len() > 6
        && buf[0] == 0x52
        && buf[1] == 0x61
        && buf[2] == 0x72
        && buf[3] == 0x21
        && buf[4] == 0x1A
        && buf[5] == 0x7
        && (buf[6] == 0x0 || buf[6] == 0x1)
}

/// Returns whether a buffer is a gzip archive.
pub fn is_gz(buf: &[u8]) -> bool {
    buf.len() > 2 && buf[0] == 0x1F && buf[1] == 0x8B && buf[2] == 0x8
}

/// Returns whether a buffer is a bzip archive.
pub fn is_bz2(buf: &[u8]) -> bool {
    buf.len() > 2 && buf[0] == 0x42 && buf[1] == 0x5A && buf[2] == 0x68
}

/// Returns whether a buffer is a 7z archive.
pub fn is_7z(buf: &[u8]) -> bool {
    buf.len() > 5
        && buf[0] == 0x37
        && buf[1] == 0x7A
        && buf[2] == 0xBC
        && buf[3] == 0xAF
        && buf[4] == 0x27
        && buf[5] == 0x1C
}

/// Returns whether a buffer is a PDF.
pub fn is_pdf(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x25 && buf[1] == 0x50 && buf[2] == 0x44 && buf[3] == 0x46
}

/// Returns whether a buffer is a SWF.
pub fn is_swf(buf: &[u8]) -> bool {
    buf.len() > 2 && (buf[0] == 0x43 || buf[0] == 0x46) && buf[1] == 0x57 && buf[2] == 0x53
}

/// Returns whether a buffer is an RTF.
pub fn is_rtf(buf: &[u8]) -> bool {
    buf.len() > 4
        && buf[0] == 0x7B
        && buf[1] == 0x5C
        && buf[2] == 0x72
        && buf[3] == 0x74
        && buf[4] == 0x66
}

/// Returns whether a buffer is a Nintendo NES ROM.
pub fn is_nes(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x4E && buf[1] == 0x45 && buf[2] == 0x53 && buf[3] == 0x1A
}

/// Returns whether a buffer is Google Chrome Extension
pub fn is_crx(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x43 && buf[1] == 0x72 && buf[2] == 0x32 && buf[3] == 0x34
}

/// Returns whether a buffer is a CAB.
pub fn is_cab(buf: &[u8]) -> bool {
    buf.len() > 3
        && ((buf[0] == 0x4D && buf[1] == 0x53 && buf[2] == 0x43 && buf[3] == 0x46)
            || (buf[0] == 0x49 && buf[1] == 0x53 && buf[2] == 0x63 && buf[3] == 0x28))
}

/// Returns whether a buffer is a eot octet stream.
pub fn is_eot(buf: &[u8]) -> bool {
    buf.len() > 35
        && buf[34] == 0x4C
        && buf[35] == 0x50
        && ((buf[8] == 0x02 && buf[9] == 0x00 && buf[10] == 0x01)
            || (buf[8] == 0x01 && buf[9] == 0x00 && buf[10] == 0x00)
            || (buf[8] == 0x02 && buf[9] == 0x00 && buf[10] == 0x02))
}

/// Returns whether a buffer is postscript.
pub fn is_ps(buf: &[u8]) -> bool {
    buf.len() > 1 && buf[0] == 0x25 && buf[1] == 0x21
}

/// Returns whether a buffer is xz archive.
pub fn is_xz(buf: &[u8]) -> bool {
    buf.len() > 5
        && buf[0] == 0xFD
        && buf[1] == 0x37
        && buf[2] == 0x7A
        && buf[3] == 0x58
        && buf[4] == 0x5A
        && buf[5] == 0x00
}

/// Returns whether a buffer is a sqlite3 database.
///
/// # Example
///
/// ```rust
/// use std::fs;
/// assert!(infer::archive::is_sqlite(&fs::read("testdata/sample.db").unwrap()));
/// ```
pub fn is_sqlite(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x53 && buf[1] == 0x51 && buf[2] == 0x4C && buf[3] == 0x69
}

/// Returns whether a buffer is a deb archive.
pub fn is_deb(buf: &[u8]) -> bool {
    buf.len() > 20
        && buf[0] == 0x21
        && buf[1] == 0x3C
        && buf[2] == 0x61
        && buf[3] == 0x72
        && buf[4] == 0x63
        && buf[5] == 0x68
        && buf[6] == 0x3E
        && buf[7] == 0x0A
        && buf[8] == 0x64
        && buf[9] == 0x65
        && buf[10] == 0x62
        && buf[11] == 0x69
        && buf[12] == 0x61
        && buf[13] == 0x6E
        && buf[14] == 0x2D
        && buf[15] == 0x62
        && buf[16] == 0x69
        && buf[17] == 0x6E
        && buf[18] == 0x61
        && buf[19] == 0x72
        && buf[20] == 0x79
}

/// Returns whether a buffer is a ar archive.
pub fn is_ar(buf: &[u8]) -> bool {
    buf.len() > 6
        && buf[0] == 0x21
        && buf[1] == 0x3C
        && buf[2] == 0x61
        && buf[3] == 0x72
        && buf[4] == 0x63
        && buf[5] == 0x68
        && buf[6] == 0x3E
}

/// Returns whether a buffer is a z archive.
pub fn is_z(buf: &[u8]) -> bool {
    buf.len() > 1 && buf[0] == 0x1F && (buf[1] == 0xA0 || buf[1] == 0x9D)
}

/// Returns whether a buffer is a lzip archive.
pub fn is_lz(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x4C && buf[1] == 0x5A && buf[2] == 0x49 && buf[3] == 0x50
}

/// Returns whether a buffer is an RPM.
pub fn is_rpm(buf: &[u8]) -> bool {
    buf.len() > 96 && buf[0] == 0xED && buf[1] == 0xAB && buf[2] == 0xEE && buf[3] == 0xDB
}

/// Returns whether a buffer is a dcm archive.
pub fn is_dcm(buf: &[u8]) -> bool {
    buf.len() > 131 && buf[128] == 0x44 && buf[129] == 0x49 && buf[130] == 0x43 && buf[131] == 0x4D
}

/// Returns whether a buffer is a Zstd archive.
pub fn is_zst(buf: &[u8]) -> bool {
    buf.len() > 3 && buf[0] == 0x28 && buf[1] == 0xB5 && buf[2] == 0x2F && buf[3] == 0xFD
}
