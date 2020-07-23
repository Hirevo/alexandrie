use super::matchers;
use super::Matcher;

#[derive(Debug, Eq, PartialEq)]
pub enum MatcherType {
    APP,
    ARCHIVE,
    AUDIO,
    DOC,
    FONT,
    IMAGE,
    VIDEO,
    CUSTOM,
}

pub fn setup(v: &mut Vec<(MatcherType, String, String, Matcher)>) {
    // TODO
    // Replace all of this with macro?

    // Order: Application, Image, Video, Audio, Font, Document, Archive

    // Application
    v.push((
        MatcherType::APP,
        "application/wasm".to_string(),
        "wasm".to_string(),
        matchers::app::is_wasm as Matcher,
    ));
    v.push((
        MatcherType::APP,
        "application/x-executable".to_string(),
        "elf".to_string(),
        matchers::app::is_elf as Matcher,
    ));
    v.push((
        MatcherType::APP,
        "application/x-msdownload".to_string(),
        "exe".to_string(),
        matchers::app::is_exe as Matcher,
    ));
    v.push((
        MatcherType::APP,
        "application/java".to_string(),
        "class".to_string(),
        matchers::app::is_java as Matcher,
    ));
    v.push((
        MatcherType::APP,
        "application/x-llvm".to_string(),
        "bc".to_string(),
        matchers::app::is_llvm as Matcher,
    ));

    // Image
    v.push((
        MatcherType::IMAGE,
        "image/jpeg".to_string(),
        "jpg".to_string(),
        matchers::image::is_jpeg as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/jp2".to_string(),
        "jp2".to_string(),
        matchers::image::is_jpeg2000 as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/png".to_string(),
        "png".to_string(),
        matchers::image::is_png as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/gif".to_string(),
        "gif".to_string(),
        matchers::image::is_gif as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/webp".to_string(),
        "webp".to_string(),
        matchers::image::is_webp as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/x-canon-cr2".to_string(),
        "cr2".to_string(),
        matchers::image::is_cr2 as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/tiff".to_string(),
        "tif".to_string(),
        matchers::image::is_tiff as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/bmp".to_string(),
        "bmp".to_string(),
        matchers::image::is_bmp as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/vnd.ms-photo".to_string(),
        "jxr".to_string(),
        matchers::image::is_jxr as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/vnd.adobe.photoshop".to_string(),
        "psd".to_string(),
        matchers::image::is_psd as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/x-icon".to_string(),
        "ico".to_string(),
        matchers::image::is_ico as Matcher,
    ));
    v.push((
        MatcherType::IMAGE,
        "image/heif".to_string(),
        "heif".to_string(),
        matchers::image::is_heif as Matcher,
    ));

    // Video
    v.push((
        MatcherType::VIDEO,
        "video/mp4".to_string(),
        "mp4".to_string(),
        matchers::video::is_mp4 as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/x-m4v".to_string(),
        "m4v".to_string(),
        matchers::video::is_m4v as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/x-matroska".to_string(),
        "mkv".to_string(),
        matchers::video::is_mkv as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/webm".to_string(),
        "webm".to_string(),
        matchers::video::is_webm as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/quicktime".to_string(),
        "mov".to_string(),
        matchers::video::is_mov as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/x-msvideo".to_string(),
        "avi".to_string(),
        matchers::video::is_avi as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/x-ms-wmv".to_string(),
        "wmv".to_string(),
        matchers::video::is_wmv as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/mpeg".to_string(),
        "mpg".to_string(),
        matchers::video::is_mpeg as Matcher,
    ));
    v.push((
        MatcherType::VIDEO,
        "video/x-flv".to_string(),
        "flv".to_string(),
        matchers::video::is_flv as Matcher,
    ));

    // Audio
    v.push((
        MatcherType::AUDIO,
        "audio/midi".to_string(),
        "midi".to_string(),
        matchers::audio::is_midi as Matcher,
    ));
    v.push((
        MatcherType::AUDIO,
        "audio/mpeg".to_string(),
        "mp3".to_string(),
        matchers::audio::is_mp3 as Matcher,
    ));
    v.push((
        MatcherType::AUDIO,
        "audio/m4a".to_string(),
        "m4a".to_string(),
        matchers::audio::is_m4a as Matcher,
    ));
    v.push((
        MatcherType::AUDIO,
        "audio/ogg".to_string(),
        "ogg".to_string(),
        matchers::audio::is_ogg as Matcher,
    ));
    v.push((
        MatcherType::AUDIO,
        "audio/x-flac".to_string(),
        "flac".to_string(),
        matchers::audio::is_flac as Matcher,
    ));
    v.push((
        MatcherType::AUDIO,
        "audio/x-wav".to_string(),
        "wav".to_string(),
        matchers::audio::is_wav as Matcher,
    ));
    v.push((
        MatcherType::AUDIO,
        "audio/amr".to_string(),
        "amr".to_string(),
        matchers::audio::is_amr as Matcher,
    ));
    v.push((
        MatcherType::AUDIO,
        "audio/aac".to_string(),
        "aac".to_string(),
        matchers::audio::is_aac as Matcher,
    ));

    // Font
    v.push((
        MatcherType::FONT,
        "application/font-woff".to_string(),
        "woff".to_string(),
        matchers::font::is_woff as Matcher,
    ));
    v.push((
        MatcherType::FONT,
        "application/font-woff".to_string(),
        "woff2".to_string(),
        matchers::font::is_woff2 as Matcher,
    ));
    v.push((
        MatcherType::FONT,
        "application/font-sfnt".to_string(),
        "ttf".to_string(),
        matchers::font::is_ttf as Matcher,
    ));
    v.push((
        MatcherType::FONT,
        "application/font-sfnt".to_string(),
        "otf".to_string(),
        matchers::font::is_otf as Matcher,
    ));

    // Document
    v.push((
        MatcherType::DOC,
        "application/msword".to_string(),
        "doc".to_string(),
        matchers::doc::is_doc as Matcher,
    ));
    v.push((
        MatcherType::DOC,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        "docx".to_string(),
        matchers::doc::is_docx as Matcher,
    ));
    v.push((
        MatcherType::DOC,
        "application/vnd.ms-excel".to_string(),
        "xls".to_string(),
        matchers::doc::is_xls as Matcher,
    ));
    v.push((
        MatcherType::DOC,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        "xlsx".to_string(),
        matchers::doc::is_xlsx as Matcher,
    ));
    v.push((
        MatcherType::DOC,
        "application/vnd.ms-powerpoint".to_string(),
        "ppt".to_string(),
        matchers::doc::is_ppt as Matcher,
    ));
    v.push((
        MatcherType::DOC,
        "application/application/vnd.openxmlformats-officedocument.presentationml.presentation"
            .to_string(),
        "pptx".to_string(),
        matchers::doc::is_pptx as Matcher,
    ));

    // Archive
    v.push((
        MatcherType::ARCHIVE,
        "application/epub+zip".to_string(),
        "epub".to_string(),
        matchers::archive::is_epub as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/zip".to_string(),
        "zip".to_string(),
        matchers::archive::is_zip as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-tar".to_string(),
        "tar".to_string(),
        matchers::archive::is_tar as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-rar-compressed".to_string(),
        "rar".to_string(),
        matchers::archive::is_rar as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/gzip".to_string(),
        "gz".to_string(),
        matchers::archive::is_gz as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-bzip2".to_string(),
        "bz2".to_string(),
        matchers::archive::is_bz2 as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-7z-compressed".to_string(),
        "7z".to_string(),
        matchers::archive::is_7z as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-xz".to_string(),
        "xz".to_string(),
        matchers::archive::is_xz as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/pdf".to_string(),
        "pdf".to_string(),
        matchers::archive::is_pdf as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-shockwave-flash".to_string(),
        "swf".to_string(),
        matchers::archive::is_swf as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/rtf".to_string(),
        "rtf".to_string(),
        matchers::archive::is_rtf as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/octet-stream".to_string(),
        "eot".to_string(),
        matchers::archive::is_eot as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/postscript".to_string(),
        "ps".to_string(),
        matchers::archive::is_ps as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-sqlite3".to_string(),
        "sqlite".to_string(),
        matchers::archive::is_sqlite as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-nintendo-nes-rom".to_string(),
        "nes".to_string(),
        matchers::archive::is_nes as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-google-chrome-extension".to_string(),
        "crx".to_string(),
        matchers::archive::is_crx as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/vnd.ms-cab-compressed".to_string(),
        "cab".to_string(),
        matchers::archive::is_cab as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-deb".to_string(),
        "deb".to_string(),
        matchers::archive::is_deb as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-unix-archive".to_string(),
        "ar".to_string(),
        matchers::archive::is_ar as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-compress".to_string(),
        "Z".to_string(),
        matchers::archive::is_z as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-lzip".to_string(),
        "lz".to_string(),
        matchers::archive::is_lz as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/x-rpm".to_string(),
        "rpm".to_string(),
        matchers::archive::is_rpm as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/dicom".to_string(),
        "dcm".to_string(),
        matchers::archive::is_dcm as Matcher,
    ));
    v.push((
        MatcherType::ARCHIVE,
        "application/zstd".to_string(),
        "zst".to_string(),
        matchers::archive::is_zst as Matcher,
    ));
}
