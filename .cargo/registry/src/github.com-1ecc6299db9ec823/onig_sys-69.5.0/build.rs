#[cfg(feature = "generate")]
extern crate bindgen;
extern crate cc;
extern crate pkg_config;

use pkg_config::Config;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// # Link Type Enumeration
///
/// Holds the different types of linking we support in this
/// script. Used to keep track of what the default link type is and
/// what override has been specified, if any, in the environment.
#[derive(Eq, PartialEq)]
enum LinkType {
    /// Static linking. This corresponds to the `static` type in Cargo.
    Static,
    /// Dynamic linking. This corresponds to the `dylib` type in Cargo.
    Dynamic,
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &LinkType::Static => "static",
                &LinkType::Dynamic => "dylib",
            }
        )
    }
}

fn env_var_bool(name: &str) -> Option<bool> {
    env::var(name)
        .ok()
        .map(|s| match &s.to_string().to_lowercase()[..] {
            "0" | "no" | "false" => false,
            _ => true,
        })
}

/// # Link Type Override
///
/// Retuns the override from the environment, if any is set.
fn link_type_override() -> Option<LinkType> {
    let dynamic_env = env_var_bool("RUSTONIG_DYNAMIC_LIBONIG").map(|b| match b {
        true => LinkType::Dynamic,
        false => LinkType::Static,
    });
    let static_env = env_var_bool("RUSTONIG_STATIC_LIBONIG").map(|b| match b {
        true => LinkType::Static,
        false => LinkType::Dynamic,
    });

    dynamic_env.or(static_env)
}

fn compile() {
    bindgen_headers("oniguruma/src/oniguruma.h");

    let mut cc = cc::Build::new();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let ref src = Path::new("oniguruma").join("src");
    let config_h = out_dir.join("config.h");

    if env_var_bool("CARGO_FEATURE_PRINT_DEBUG").unwrap_or(false) {
        cc.define("ONIG_DEBUG_PARSE", Some("1"));
        cc.define("ONIG_DEBUG_COMPILE", Some("1"));
        cc.define("ONIG_DEBUG_SEARCH", Some("1"));
        cc.define("ONIG_DEBUG_MATCH", Some("1"));
    }

    if !src.exists() {
        panic!(
            "Unable to find source files in {}. Is oniguruma submodule checked out?\n\
             Try git submodule init; git submodule update",
            src.display()
        );
    }

    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let bits = env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();
    if os == "windows" {
        fs::copy(src.join(format!("config.h.win{}", bits)), config_h)
            .expect("Can't copy config.h.win??");
    } else {
        let family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap();
        if family == "unix" {
            cc.define("HAVE_UNISTD_H", Some("1"));
            cc.define("HAVE_SYS_TYPES_H", Some("1"));
            cc.define("HAVE_SYS_TIME_H", Some("1"));
        }

        // Can't use size_of::<c_long>(), because it'd refer to build arch, not target arch.
        // so instead assume it's a non-exotic target (LP32/LP64).
        fs::write(
            config_h,
            format!(
                "
            #define HAVE_PROTOTYPES 1
            #define STDC_HEADERS 1
            #define HAVE_STRING_H 1
            #define HAVE_STDARG_H 1
            #define HAVE_STDLIB_H 1
            #define HAVE_LIMITS_H 1
            #define HAVE_INTTYPES_H 1
            #define SIZEOF_INT 4
            #define SIZEOF_SHORT 2
            #define SIZEOF_LONG {}
        ",
                if bits == "64" { "8" } else { "4" }
            ),
        )
        .expect("Can't write config.h to OUT_DIR");
    }

    cc.include(out_dir); // Read config.h from there
    cc.include(src);

    let files = [
        "regexec.c",
        "regerror.c",
        "regparse.c",
        "regext.c",
        "regcomp.c",
        "reggnu.c",
        "regenc.c",
        "regsyntax.c",
        "regtrav.c",
        "regversion.c",
        "st.c",
        "onig_init.c",
        "unicode.c",
        "ascii.c",
        "utf8.c",
        "utf16_be.c",
        "utf16_le.c",
        "utf32_be.c",
        "utf32_le.c",
        "euc_jp.c",
        "sjis.c",
        "iso8859_1.c",
        "iso8859_2.c",
        "iso8859_3.c",
        "iso8859_4.c",
        "iso8859_5.c",
        "iso8859_6.c",
        "iso8859_7.c",
        "iso8859_8.c",
        "iso8859_9.c",
        "iso8859_10.c",
        "iso8859_11.c",
        "iso8859_13.c",
        "iso8859_14.c",
        "iso8859_15.c",
        "iso8859_16.c",
        "euc_tw.c",
        "euc_kr.c",
        "big5.c",
        "gb18030.c",
        "koi8_r.c",
        "cp1251.c",
        "euc_jp_prop.c",
        "sjis_prop.c",
        "unicode_unfold_key.c",
        "unicode_fold1_key.c",
        "unicode_fold2_key.c",
        "unicode_fold3_key.c",
    ];
    for file in files.iter() {
        cc.file(src.join(file));
    }

    if cfg!(feature = "posix-api") {
        cc.file(src.join("regposix.c"));
        cc.file(src.join("regposerr.c"));
    }

    cc.warnings(false); // not actionable by the end user
    cc.compile("onig");
}

fn bindgen_headers(_path: &str) {
    #[cfg(feature = "generate")]
    {
        let bindings = bindgen::Builder::default()
            .header(_path)
            .derive_eq(true)
            .layout_tests(false)
            .generate()
            .expect("bindgen");
        let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR");
        let out_path = Path::new(&out_dir);
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}

pub fn main() {
    let link_type = link_type_override();
    let require_pkg_config = env_var_bool("RUSTONIG_SYSTEM_LIBONIG").unwrap_or(false);

    if require_pkg_config || link_type == Some(LinkType::Dynamic) {
        let mut conf = Config::new();
        // dynamically-generated headers can work with an older version
        // pre-generated headers are for the latest
        conf.atleast_version(if cfg!(feature = "generate") {"6.8.0"} else {"6.9.3"});
        if link_type == Some(LinkType::Static) {
            conf.statik(true);
        }
        match conf.probe("oniguruma") {
            Ok(lib) => {
                for path in &lib.include_paths {
                    let header = path.join("oniguruma.h");
                    if header.exists() {
                        bindgen_headers(&header.display().to_string());
                        return
                    }
                }
                if require_pkg_config {
                    panic!("Unable to find oniguruma.h in include paths from pkg-config: {:?}", lib.include_paths);
                }
            },
            Err(ref err) if require_pkg_config => {
                panic!("Unable to find oniguruma in pkg-config, and RUSTONIG_SYSTEM_LIBONIG is set: {}", err);
            }
            _ => {}
        }
    }

    compile();
}
