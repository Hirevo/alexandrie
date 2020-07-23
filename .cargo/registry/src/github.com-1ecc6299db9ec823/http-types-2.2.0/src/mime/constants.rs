use super::ParamKind;
use crate::Mime;

macro_rules! utf8_mime_const {
    ($name:ident, $desc:expr, $base:expr, $sub:expr) => {
        mime_const!(
            with_params,
            $name,
            $desc,
            $base,
            $sub,
            Some(ParamKind::Utf8),
            ";charset=utf-8"
        );
    };
}
macro_rules! mime_const {
    ($name:ident, $desc:expr, $base:expr, $sub:expr) => {
        mime_const!(with_params, $name, $desc, $base, $sub, None, "");
    };

    (with_params, $name:ident, $desc:expr, $base:expr, $sub:expr, $params:expr, $doccomment:expr) => {
        mime_const!(doc_expanded, $name, $desc, $base, $sub, $params,
             concat!(
                "Content-Type for ",
                $desc,
                ".\n\n# Mime Type\n\n```text\n",
                $base, "/", $sub, $doccomment, "\n```")
        );
    };

    (doc_expanded, $name:ident, $desc:expr, $base:expr, $sub:expr, $params:expr, $doccomment:expr) => {
        #[doc = $doccomment]
        pub const $name: Mime = Mime {
            essence: String::new(),
            basetype: String::new(),
            subtype: String::new(),
            params: $params,
            static_essence: Some(concat!($base, "/", $sub)),
            static_basetype: Some($base),
            static_subtype: Some($sub),
        };
    };
}

utf8_mime_const!(JAVASCRIPT, "JavaScript", "application", "javascript");
utf8_mime_const!(CSS, "CSS", "text", "css");
utf8_mime_const!(HTML, "HTML", "text", "html");
utf8_mime_const!(PLAIN, "Plain text", "text", "plain");
mime_const!(ANY, "matching anything", "*", "*");
mime_const!(JSON, "JSON", "application", "json");
mime_const!(SVG, "SVG", "image", "svg+xml");
mime_const!(PNG, "PNG images", "image", "png");
mime_const!(JPEG, "JPEG images", "image", "jpeg");
mime_const!(SSE, "Server Sent Events", "text", "event-stream");
mime_const!(BYTE_STREAM, "byte streams", "application", "octet-stream");
mime_const!(FORM, "forms", "application", "x-www-form-urlencoded");
mime_const!(MULTIPART_FORM, "multipart forms", "multipart", "form-data");
mime_const!(WASM, "webassembly", "application", "wasm");
// There are multiple `.ico` mime types known, but `image/x-icon`
// is what most browser use. See:
// https://en.wikipedia.org/wiki/ICO_%28file_format%29#MIME_type
mime_const!(ICO, "ICO icons", "image", "x-icon");
