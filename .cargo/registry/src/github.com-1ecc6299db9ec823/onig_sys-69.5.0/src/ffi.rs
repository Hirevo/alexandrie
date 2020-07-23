/* generated with bindgen oniguruma/src/oniguruma.h --with-derive-eq --no-layout-tests --distrust-clang-mangling > src/ffi.rs */

pub const ONIGURUMA_VERSION_MAJOR: u32 = 6;
pub const ONIGURUMA_VERSION_MINOR: u32 = 9;
pub const ONIGURUMA_VERSION_TEENY: u32 = 5;
pub const ONIGURUMA_VERSION_INT: u32 = 60905;
pub const ONIGENC_CASE_FOLD_TURKISH_AZERI: u32 = 1048576;
pub const INTERNAL_ONIGENC_CASE_FOLD_MULTI_CHAR: u32 = 1073741824;
pub const ONIGENC_CASE_FOLD_MIN: u32 = 1073741824;
pub const ONIGENC_MAX_COMP_CASE_FOLD_CODE_LEN: u32 = 3;
pub const ONIGENC_GET_CASE_FOLD_CODES_MAX_NUM: u32 = 13;
pub const ONIGENC_CODE_TO_MBC_MAXLEN: u32 = 7;
pub const ONIGENC_MBC_CASE_FOLD_MAXLEN: u32 = 18;
pub const ONIG_NREGION: u32 = 10;
pub const ONIG_MAX_CAPTURE_NUM: u32 = 2147483647;
pub const ONIG_MAX_BACKREF_NUM: u32 = 1000;
pub const ONIG_MAX_REPEAT_NUM: u32 = 100000;
pub const ONIG_MAX_MULTI_BYTE_RANGES_NUM: u32 = 10000;
pub const ONIG_MAX_ERROR_MESSAGE_LEN: u32 = 90;
pub const ONIG_OPTION_NONE: u32 = 0;
pub const ONIG_OPTION_IGNORECASE: u32 = 1;
pub const ONIG_OPTION_EXTEND: u32 = 2;
pub const ONIG_OPTION_MULTILINE: u32 = 4;
pub const ONIG_OPTION_SINGLELINE: u32 = 8;
pub const ONIG_OPTION_FIND_LONGEST: u32 = 16;
pub const ONIG_OPTION_FIND_NOT_EMPTY: u32 = 32;
pub const ONIG_OPTION_NEGATE_SINGLELINE: u32 = 64;
pub const ONIG_OPTION_DONT_CAPTURE_GROUP: u32 = 128;
pub const ONIG_OPTION_CAPTURE_GROUP: u32 = 256;
pub const ONIG_OPTION_NOTBOL: u32 = 512;
pub const ONIG_OPTION_NOTEOL: u32 = 1024;
pub const ONIG_OPTION_POSIX_REGION: u32 = 2048;
pub const ONIG_OPTION_CHECK_VALIDITY_OF_STRING: u32 = 4096;
pub const ONIG_OPTION_WORD_IS_ASCII: u32 = 65536;
pub const ONIG_OPTION_DIGIT_IS_ASCII: u32 = 131072;
pub const ONIG_OPTION_SPACE_IS_ASCII: u32 = 262144;
pub const ONIG_OPTION_POSIX_IS_ASCII: u32 = 524288;
pub const ONIG_OPTION_TEXT_SEGMENT_EXTENDED_GRAPHEME_CLUSTER: u32 = 1048576;
pub const ONIG_OPTION_TEXT_SEGMENT_WORD: u32 = 2097152;
pub const ONIG_OPTION_MAXBIT: u32 = 2097152;
pub const ONIG_SYN_OP_VARIABLE_META_CHARACTERS: u32 = 1;
pub const ONIG_SYN_OP_DOT_ANYCHAR: u32 = 2;
pub const ONIG_SYN_OP_ASTERISK_ZERO_INF: u32 = 4;
pub const ONIG_SYN_OP_ESC_ASTERISK_ZERO_INF: u32 = 8;
pub const ONIG_SYN_OP_PLUS_ONE_INF: u32 = 16;
pub const ONIG_SYN_OP_ESC_PLUS_ONE_INF: u32 = 32;
pub const ONIG_SYN_OP_QMARK_ZERO_ONE: u32 = 64;
pub const ONIG_SYN_OP_ESC_QMARK_ZERO_ONE: u32 = 128;
pub const ONIG_SYN_OP_BRACE_INTERVAL: u32 = 256;
pub const ONIG_SYN_OP_ESC_BRACE_INTERVAL: u32 = 512;
pub const ONIG_SYN_OP_VBAR_ALT: u32 = 1024;
pub const ONIG_SYN_OP_ESC_VBAR_ALT: u32 = 2048;
pub const ONIG_SYN_OP_LPAREN_SUBEXP: u32 = 4096;
pub const ONIG_SYN_OP_ESC_LPAREN_SUBEXP: u32 = 8192;
pub const ONIG_SYN_OP_ESC_AZ_BUF_ANCHOR: u32 = 16384;
pub const ONIG_SYN_OP_ESC_CAPITAL_G_BEGIN_ANCHOR: u32 = 32768;
pub const ONIG_SYN_OP_DECIMAL_BACKREF: u32 = 65536;
pub const ONIG_SYN_OP_BRACKET_CC: u32 = 131072;
pub const ONIG_SYN_OP_ESC_W_WORD: u32 = 262144;
pub const ONIG_SYN_OP_ESC_LTGT_WORD_BEGIN_END: u32 = 524288;
pub const ONIG_SYN_OP_ESC_B_WORD_BOUND: u32 = 1048576;
pub const ONIG_SYN_OP_ESC_S_WHITE_SPACE: u32 = 2097152;
pub const ONIG_SYN_OP_ESC_D_DIGIT: u32 = 4194304;
pub const ONIG_SYN_OP_LINE_ANCHOR: u32 = 8388608;
pub const ONIG_SYN_OP_POSIX_BRACKET: u32 = 16777216;
pub const ONIG_SYN_OP_QMARK_NON_GREEDY: u32 = 33554432;
pub const ONIG_SYN_OP_ESC_CONTROL_CHARS: u32 = 67108864;
pub const ONIG_SYN_OP_ESC_C_CONTROL: u32 = 134217728;
pub const ONIG_SYN_OP_ESC_OCTAL3: u32 = 268435456;
pub const ONIG_SYN_OP_ESC_X_HEX2: u32 = 536870912;
pub const ONIG_SYN_OP_ESC_X_BRACE_HEX8: u32 = 1073741824;
pub const ONIG_SYN_OP_ESC_O_BRACE_OCTAL: u32 = 2147483648;
pub const ONIG_SYN_OP2_ESC_CAPITAL_Q_QUOTE: u32 = 1;
pub const ONIG_SYN_OP2_QMARK_GROUP_EFFECT: u32 = 2;
pub const ONIG_SYN_OP2_OPTION_PERL: u32 = 4;
pub const ONIG_SYN_OP2_OPTION_RUBY: u32 = 8;
pub const ONIG_SYN_OP2_PLUS_POSSESSIVE_REPEAT: u32 = 16;
pub const ONIG_SYN_OP2_PLUS_POSSESSIVE_INTERVAL: u32 = 32;
pub const ONIG_SYN_OP2_CCLASS_SET_OP: u32 = 64;
pub const ONIG_SYN_OP2_QMARK_LT_NAMED_GROUP: u32 = 128;
pub const ONIG_SYN_OP2_ESC_K_NAMED_BACKREF: u32 = 256;
pub const ONIG_SYN_OP2_ESC_G_SUBEXP_CALL: u32 = 512;
pub const ONIG_SYN_OP2_ATMARK_CAPTURE_HISTORY: u32 = 1024;
pub const ONIG_SYN_OP2_ESC_CAPITAL_C_BAR_CONTROL: u32 = 2048;
pub const ONIG_SYN_OP2_ESC_CAPITAL_M_BAR_META: u32 = 4096;
pub const ONIG_SYN_OP2_ESC_V_VTAB: u32 = 8192;
pub const ONIG_SYN_OP2_ESC_U_HEX4: u32 = 16384;
pub const ONIG_SYN_OP2_ESC_GNU_BUF_ANCHOR: u32 = 32768;
pub const ONIG_SYN_OP2_ESC_P_BRACE_CHAR_PROPERTY: u32 = 65536;
pub const ONIG_SYN_OP2_ESC_P_BRACE_CIRCUMFLEX_NOT: u32 = 131072;
pub const ONIG_SYN_OP2_ESC_H_XDIGIT: u32 = 524288;
pub const ONIG_SYN_OP2_INEFFECTIVE_ESCAPE: u32 = 1048576;
pub const ONIG_SYN_OP2_QMARK_LPAREN_IF_ELSE: u32 = 2097152;
pub const ONIG_SYN_OP2_ESC_CAPITAL_K_KEEP: u32 = 4194304;
pub const ONIG_SYN_OP2_ESC_CAPITAL_R_GENERAL_NEWLINE: u32 = 8388608;
pub const ONIG_SYN_OP2_ESC_CAPITAL_N_O_SUPER_DOT: u32 = 16777216;
pub const ONIG_SYN_OP2_QMARK_TILDE_ABSENT_GROUP: u32 = 33554432;
pub const ONIG_SYN_OP2_ESC_X_Y_GRAPHEME_CLUSTER: u32 = 67108864;
pub const ONIG_SYN_OP2_ESC_X_Y_TEXT_SEGMENT: u32 = 67108864;
pub const ONIG_SYN_OP2_QMARK_PERL_SUBEXP_CALL: u32 = 134217728;
pub const ONIG_SYN_OP2_QMARK_BRACE_CALLOUT_CONTENTS: u32 = 268435456;
pub const ONIG_SYN_OP2_ASTERISK_CALLOUT_NAME: u32 = 536870912;
pub const ONIG_SYN_OP2_OPTION_ONIGURUMA: u32 = 1073741824;
pub const ONIG_SYN_CONTEXT_INDEP_ANCHORS: u32 = 2147483648;
pub const ONIG_SYN_CONTEXT_INDEP_REPEAT_OPS: u32 = 1;
pub const ONIG_SYN_CONTEXT_INVALID_REPEAT_OPS: u32 = 2;
pub const ONIG_SYN_ALLOW_UNMATCHED_CLOSE_SUBEXP: u32 = 4;
pub const ONIG_SYN_ALLOW_INVALID_INTERVAL: u32 = 8;
pub const ONIG_SYN_ALLOW_INTERVAL_LOW_ABBREV: u32 = 16;
pub const ONIG_SYN_STRICT_CHECK_BACKREF: u32 = 32;
pub const ONIG_SYN_DIFFERENT_LEN_ALT_LOOK_BEHIND: u32 = 64;
pub const ONIG_SYN_CAPTURE_ONLY_NAMED_GROUP: u32 = 128;
pub const ONIG_SYN_ALLOW_MULTIPLEX_DEFINITION_NAME: u32 = 256;
pub const ONIG_SYN_FIXED_INTERVAL_IS_GREEDY_ONLY: u32 = 512;
pub const ONIG_SYN_ISOLATED_OPTION_CONTINUE_BRANCH: u32 = 1024;
pub const ONIG_SYN_VARIABLE_LEN_LOOK_BEHIND: u32 = 2048;
pub const ONIG_SYN_NOT_NEWLINE_IN_NEGATIVE_CC: u32 = 1048576;
pub const ONIG_SYN_BACKSLASH_ESCAPE_IN_CC: u32 = 2097152;
pub const ONIG_SYN_ALLOW_EMPTY_RANGE_IN_CC: u32 = 4194304;
pub const ONIG_SYN_ALLOW_DOUBLE_RANGE_OP_IN_CC: u32 = 8388608;
pub const ONIG_SYN_ALLOW_INVALID_CODE_END_OF_RANGE_IN_CC: u32 = 67108864;
pub const ONIG_SYN_WARN_CC_OP_NOT_ESCAPED: u32 = 16777216;
pub const ONIG_SYN_WARN_REDUNDANT_NESTED_REPEAT: u32 = 33554432;
pub const ONIG_META_CHAR_ESCAPE: u32 = 0;
pub const ONIG_META_CHAR_ANYCHAR: u32 = 1;
pub const ONIG_META_CHAR_ANYTIME: u32 = 2;
pub const ONIG_META_CHAR_ZERO_OR_ONE_TIME: u32 = 3;
pub const ONIG_META_CHAR_ONE_OR_MORE_TIME: u32 = 4;
pub const ONIG_META_CHAR_ANYCHAR_ANYTIME: u32 = 5;
pub const ONIG_INEFFECTIVE_META_CHAR: u32 = 0;
pub const ONIG_NORMAL: u32 = 0;
pub const ONIG_MISMATCH: i32 = -1;
pub const ONIG_NO_SUPPORT_CONFIG: i32 = -2;
pub const ONIG_ABORT: i32 = -3;
pub const ONIGERR_MEMORY: i32 = -5;
pub const ONIGERR_TYPE_BUG: i32 = -6;
pub const ONIGERR_PARSER_BUG: i32 = -11;
pub const ONIGERR_STACK_BUG: i32 = -12;
pub const ONIGERR_UNDEFINED_BYTECODE: i32 = -13;
pub const ONIGERR_UNEXPECTED_BYTECODE: i32 = -14;
pub const ONIGERR_MATCH_STACK_LIMIT_OVER: i32 = -15;
pub const ONIGERR_PARSE_DEPTH_LIMIT_OVER: i32 = -16;
pub const ONIGERR_RETRY_LIMIT_IN_MATCH_OVER: i32 = -17;
pub const ONIGERR_RETRY_LIMIT_IN_SEARCH_OVER: i32 = -18;
pub const ONIGERR_DEFAULT_ENCODING_IS_NOT_SETTED: i32 = -21;
pub const ONIGERR_SPECIFIED_ENCODING_CANT_CONVERT_TO_WIDE_CHAR: i32 = -22;
pub const ONIGERR_FAIL_TO_INITIALIZE: i32 = -23;
pub const ONIGERR_INVALID_ARGUMENT: i32 = -30;
pub const ONIGERR_END_PATTERN_AT_LEFT_BRACE: i32 = -100;
pub const ONIGERR_END_PATTERN_AT_LEFT_BRACKET: i32 = -101;
pub const ONIGERR_EMPTY_CHAR_CLASS: i32 = -102;
pub const ONIGERR_PREMATURE_END_OF_CHAR_CLASS: i32 = -103;
pub const ONIGERR_END_PATTERN_AT_ESCAPE: i32 = -104;
pub const ONIGERR_END_PATTERN_AT_META: i32 = -105;
pub const ONIGERR_END_PATTERN_AT_CONTROL: i32 = -106;
pub const ONIGERR_META_CODE_SYNTAX: i32 = -108;
pub const ONIGERR_CONTROL_CODE_SYNTAX: i32 = -109;
pub const ONIGERR_CHAR_CLASS_VALUE_AT_END_OF_RANGE: i32 = -110;
pub const ONIGERR_CHAR_CLASS_VALUE_AT_START_OF_RANGE: i32 = -111;
pub const ONIGERR_UNMATCHED_RANGE_SPECIFIER_IN_CHAR_CLASS: i32 = -112;
pub const ONIGERR_TARGET_OF_REPEAT_OPERATOR_NOT_SPECIFIED: i32 = -113;
pub const ONIGERR_TARGET_OF_REPEAT_OPERATOR_INVALID: i32 = -114;
pub const ONIGERR_NESTED_REPEAT_OPERATOR: i32 = -115;
pub const ONIGERR_UNMATCHED_CLOSE_PARENTHESIS: i32 = -116;
pub const ONIGERR_END_PATTERN_WITH_UNMATCHED_PARENTHESIS: i32 = -117;
pub const ONIGERR_END_PATTERN_IN_GROUP: i32 = -118;
pub const ONIGERR_UNDEFINED_GROUP_OPTION: i32 = -119;
pub const ONIGERR_INVALID_POSIX_BRACKET_TYPE: i32 = -121;
pub const ONIGERR_INVALID_LOOK_BEHIND_PATTERN: i32 = -122;
pub const ONIGERR_INVALID_REPEAT_RANGE_PATTERN: i32 = -123;
pub const ONIGERR_TOO_BIG_NUMBER: i32 = -200;
pub const ONIGERR_TOO_BIG_NUMBER_FOR_REPEAT_RANGE: i32 = -201;
pub const ONIGERR_UPPER_SMALLER_THAN_LOWER_IN_REPEAT_RANGE: i32 = -202;
pub const ONIGERR_EMPTY_RANGE_IN_CHAR_CLASS: i32 = -203;
pub const ONIGERR_MISMATCH_CODE_LENGTH_IN_CLASS_RANGE: i32 = -204;
pub const ONIGERR_TOO_MANY_MULTI_BYTE_RANGES: i32 = -205;
pub const ONIGERR_TOO_SHORT_MULTI_BYTE_STRING: i32 = -206;
pub const ONIGERR_TOO_BIG_BACKREF_NUMBER: i32 = -207;
pub const ONIGERR_INVALID_BACKREF: i32 = -208;
pub const ONIGERR_NUMBERED_BACKREF_OR_CALL_NOT_ALLOWED: i32 = -209;
pub const ONIGERR_TOO_MANY_CAPTURES: i32 = -210;
pub const ONIGERR_TOO_LONG_WIDE_CHAR_VALUE: i32 = -212;
pub const ONIGERR_EMPTY_GROUP_NAME: i32 = -214;
pub const ONIGERR_INVALID_GROUP_NAME: i32 = -215;
pub const ONIGERR_INVALID_CHAR_IN_GROUP_NAME: i32 = -216;
pub const ONIGERR_UNDEFINED_NAME_REFERENCE: i32 = -217;
pub const ONIGERR_UNDEFINED_GROUP_REFERENCE: i32 = -218;
pub const ONIGERR_MULTIPLEX_DEFINED_NAME: i32 = -219;
pub const ONIGERR_MULTIPLEX_DEFINITION_NAME_CALL: i32 = -220;
pub const ONIGERR_NEVER_ENDING_RECURSION: i32 = -221;
pub const ONIGERR_GROUP_NUMBER_OVER_FOR_CAPTURE_HISTORY: i32 = -222;
pub const ONIGERR_INVALID_CHAR_PROPERTY_NAME: i32 = -223;
pub const ONIGERR_INVALID_IF_ELSE_SYNTAX: i32 = -224;
pub const ONIGERR_INVALID_ABSENT_GROUP_PATTERN: i32 = -225;
pub const ONIGERR_INVALID_ABSENT_GROUP_GENERATOR_PATTERN: i32 = -226;
pub const ONIGERR_INVALID_CALLOUT_PATTERN: i32 = -227;
pub const ONIGERR_INVALID_CALLOUT_NAME: i32 = -228;
pub const ONIGERR_UNDEFINED_CALLOUT_NAME: i32 = -229;
pub const ONIGERR_INVALID_CALLOUT_BODY: i32 = -230;
pub const ONIGERR_INVALID_CALLOUT_TAG_NAME: i32 = -231;
pub const ONIGERR_INVALID_CALLOUT_ARG: i32 = -232;
pub const ONIGERR_INVALID_CODE_POINT_VALUE: i32 = -400;
pub const ONIGERR_INVALID_WIDE_CHAR_VALUE: i32 = -400;
pub const ONIGERR_TOO_BIG_WIDE_CHAR_VALUE: i32 = -401;
pub const ONIGERR_NOT_SUPPORTED_ENCODING_COMBINATION: i32 = -402;
pub const ONIGERR_INVALID_COMBINATION_OF_OPTIONS: i32 = -403;
pub const ONIGERR_TOO_MANY_USER_DEFINED_OBJECTS: i32 = -404;
pub const ONIGERR_TOO_LONG_PROPERTY_NAME: i32 = -405;
pub const ONIGERR_LIBRARY_IS_NOT_INITIALIZED: i32 = -500;
pub const ONIG_MAX_CAPTURE_HISTORY_GROUP: u32 = 31;
pub const ONIG_TRAVERSE_CALLBACK_AT_FIRST: u32 = 1;
pub const ONIG_TRAVERSE_CALLBACK_AT_LAST: u32 = 2;
pub const ONIG_TRAVERSE_CALLBACK_AT_BOTH: u32 = 3;
pub const ONIG_REGION_NOTPOS: i32 = -1;
pub const ONIG_CHAR_TABLE_SIZE: u32 = 256;
pub const ONIG_NON_NAME_ID: i32 = -1;
pub const ONIG_NON_CALLOUT_NUM: u32 = 0;
pub const ONIG_CALLOUT_MAX_ARGS_NUM: u32 = 4;
pub const ONIG_CALLOUT_DATA_SLOT_NUM: u32 = 5;
pub type OnigCodePoint = ::std::os::raw::c_uint;
pub type OnigUChar = ::std::os::raw::c_uchar;
pub type OnigCtype = ::std::os::raw::c_uint;
pub type OnigLen = ::std::os::raw::c_uint;
pub type OnigCaseFoldType = ::std::os::raw::c_uint;
extern "C" {
    pub static mut OnigDefaultCaseFoldFlag: OnigCaseFoldType;
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigCaseFoldCodeItem {
    pub byte_len: ::std::os::raw::c_int,
    pub code_len: ::std::os::raw::c_int,
    pub code: [OnigCodePoint; 3usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigMetaCharTableType {
    pub esc: OnigCodePoint,
    pub anychar: OnigCodePoint,
    pub anytime: OnigCodePoint,
    pub zero_or_one_time: OnigCodePoint,
    pub one_or_more_time: OnigCodePoint,
    pub anychar_anytime: OnigCodePoint,
}
pub type OnigApplyAllCaseFoldFunc = ::std::option::Option<
    unsafe extern "C" fn(
        from: OnigCodePoint,
        to: *mut OnigCodePoint,
        to_len: ::std::os::raw::c_int,
        arg: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigEncodingTypeST {
    pub mbc_enc_len:
        ::std::option::Option<unsafe extern "C" fn(p: *const OnigUChar) -> ::std::os::raw::c_int>,
    pub name: *const ::std::os::raw::c_char,
    pub max_enc_len: ::std::os::raw::c_int,
    pub min_enc_len: ::std::os::raw::c_int,
    pub is_mbc_newline: ::std::option::Option<
        unsafe extern "C" fn(p: *const OnigUChar, end: *const OnigUChar) -> ::std::os::raw::c_int,
    >,
    pub mbc_to_code: ::std::option::Option<
        unsafe extern "C" fn(p: *const OnigUChar, end: *const OnigUChar) -> OnigCodePoint,
    >,
    pub code_to_mbclen:
        ::std::option::Option<unsafe extern "C" fn(code: OnigCodePoint) -> ::std::os::raw::c_int>,
    pub code_to_mbc: ::std::option::Option<
        unsafe extern "C" fn(code: OnigCodePoint, buf: *mut OnigUChar) -> ::std::os::raw::c_int,
    >,
    pub mbc_case_fold: ::std::option::Option<
        unsafe extern "C" fn(
            flag: OnigCaseFoldType,
            pp: *mut *const OnigUChar,
            end: *const OnigUChar,
            to: *mut OnigUChar,
        ) -> ::std::os::raw::c_int,
    >,
    pub apply_all_case_fold: ::std::option::Option<
        unsafe extern "C" fn(
            flag: OnigCaseFoldType,
            f: OnigApplyAllCaseFoldFunc,
            arg: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub get_case_fold_codes_by_str: ::std::option::Option<
        unsafe extern "C" fn(
            flag: OnigCaseFoldType,
            p: *const OnigUChar,
            end: *const OnigUChar,
            acs: *mut OnigCaseFoldCodeItem,
        ) -> ::std::os::raw::c_int,
    >,
    pub property_name_to_ctype: ::std::option::Option<
        unsafe extern "C" fn(
            enc: *mut OnigEncodingTypeST,
            p: *mut OnigUChar,
            end: *mut OnigUChar,
        ) -> ::std::os::raw::c_int,
    >,
    pub is_code_ctype: ::std::option::Option<
        unsafe extern "C" fn(code: OnigCodePoint, ctype: OnigCtype) -> ::std::os::raw::c_int,
    >,
    pub get_ctype_code_range: ::std::option::Option<
        unsafe extern "C" fn(
            ctype: OnigCtype,
            sb_out: *mut OnigCodePoint,
            ranges: *mut *const OnigCodePoint,
        ) -> ::std::os::raw::c_int,
    >,
    pub left_adjust_char_head: ::std::option::Option<
        unsafe extern "C" fn(start: *const OnigUChar, p: *const OnigUChar) -> *mut OnigUChar,
    >,
    pub is_allowed_reverse_match: ::std::option::Option<
        unsafe extern "C" fn(p: *const OnigUChar, end: *const OnigUChar) -> ::std::os::raw::c_int,
    >,
    pub init: ::std::option::Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub is_initialized: ::std::option::Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub is_valid_mbc_string: ::std::option::Option<
        unsafe extern "C" fn(s: *const OnigUChar, end: *const OnigUChar) -> ::std::os::raw::c_int,
    >,
    pub flag: ::std::os::raw::c_uint,
    pub sb_range: OnigCodePoint,
    pub index: ::std::os::raw::c_int,
}
pub type OnigEncodingType = OnigEncodingTypeST;
pub type OnigEncoding = *mut OnigEncodingType;
extern "C" {
    pub static mut OnigEncodingASCII: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_1: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_2: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_3: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_4: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_5: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_6: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_7: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_8: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_9: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_10: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_11: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_13: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_14: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_15: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingISO_8859_16: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingUTF8: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingUTF16_BE: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingUTF16_LE: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingUTF32_BE: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingUTF32_LE: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingEUC_JP: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingEUC_TW: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingEUC_KR: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingEUC_CN: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingSJIS: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingKOI8: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingKOI8_R: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingCP1251: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingBIG5: OnigEncodingType;
}
extern "C" {
    pub static mut OnigEncodingGB18030: OnigEncodingType;
}
pub const OnigEncCtype_ONIGENC_CTYPE_NEWLINE: OnigEncCtype = 0;
pub const OnigEncCtype_ONIGENC_CTYPE_ALPHA: OnigEncCtype = 1;
pub const OnigEncCtype_ONIGENC_CTYPE_BLANK: OnigEncCtype = 2;
pub const OnigEncCtype_ONIGENC_CTYPE_CNTRL: OnigEncCtype = 3;
pub const OnigEncCtype_ONIGENC_CTYPE_DIGIT: OnigEncCtype = 4;
pub const OnigEncCtype_ONIGENC_CTYPE_GRAPH: OnigEncCtype = 5;
pub const OnigEncCtype_ONIGENC_CTYPE_LOWER: OnigEncCtype = 6;
pub const OnigEncCtype_ONIGENC_CTYPE_PRINT: OnigEncCtype = 7;
pub const OnigEncCtype_ONIGENC_CTYPE_PUNCT: OnigEncCtype = 8;
pub const OnigEncCtype_ONIGENC_CTYPE_SPACE: OnigEncCtype = 9;
pub const OnigEncCtype_ONIGENC_CTYPE_UPPER: OnigEncCtype = 10;
pub const OnigEncCtype_ONIGENC_CTYPE_XDIGIT: OnigEncCtype = 11;
pub const OnigEncCtype_ONIGENC_CTYPE_WORD: OnigEncCtype = 12;
pub const OnigEncCtype_ONIGENC_CTYPE_ALNUM: OnigEncCtype = 13;
pub const OnigEncCtype_ONIGENC_CTYPE_ASCII: OnigEncCtype = 14;
pub type OnigEncCtype = u32;
extern "C" {
    pub fn onigenc_step_back(
        enc: OnigEncoding,
        start: *const OnigUChar,
        s: *const OnigUChar,
        n: ::std::os::raw::c_int,
    ) -> *mut OnigUChar;
}
extern "C" {
    pub fn onigenc_init() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_initialize_encoding(enc: OnigEncoding) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onigenc_set_default_encoding(enc: OnigEncoding) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onigenc_get_default_encoding() -> OnigEncoding;
}
extern "C" {
    pub fn onigenc_set_default_caseconv_table(table: *const OnigUChar);
}
extern "C" {
    pub fn onigenc_get_right_adjust_char_head_with_prev(
        enc: OnigEncoding,
        start: *const OnigUChar,
        s: *const OnigUChar,
        prev: *mut *const OnigUChar,
    ) -> *mut OnigUChar;
}
extern "C" {
    pub fn onigenc_get_prev_char_head(
        enc: OnigEncoding,
        start: *const OnigUChar,
        s: *const OnigUChar,
    ) -> *mut OnigUChar;
}
extern "C" {
    pub fn onigenc_get_left_adjust_char_head(
        enc: OnigEncoding,
        start: *const OnigUChar,
        s: *const OnigUChar,
    ) -> *mut OnigUChar;
}
extern "C" {
    pub fn onigenc_get_right_adjust_char_head(
        enc: OnigEncoding,
        start: *const OnigUChar,
        s: *const OnigUChar,
    ) -> *mut OnigUChar;
}
extern "C" {
    pub fn onigenc_strlen(
        enc: OnigEncoding,
        p: *const OnigUChar,
        end: *const OnigUChar,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onigenc_strlen_null(enc: OnigEncoding, p: *const OnigUChar) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onigenc_str_bytelen_null(
        enc: OnigEncoding,
        p: *const OnigUChar,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onigenc_is_valid_mbc_string(
        enc: OnigEncoding,
        s: *const OnigUChar,
        end: *const OnigUChar,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onigenc_strdup(
        enc: OnigEncoding,
        s: *const OnigUChar,
        end: *const OnigUChar,
    ) -> *mut OnigUChar;
}
pub type OnigOptionType = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigSyntaxType {
    pub op: ::std::os::raw::c_uint,
    pub op2: ::std::os::raw::c_uint,
    pub behavior: ::std::os::raw::c_uint,
    pub options: OnigOptionType,
    pub meta_char_table: OnigMetaCharTableType,
}
extern "C" {
    pub static mut OnigSyntaxASIS: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxPosixBasic: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxPosixExtended: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxEmacs: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxGrep: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxGnuRegex: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxJava: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxPerl: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxPerl_NG: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxRuby: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigSyntaxOniguruma: OnigSyntaxType;
}
extern "C" {
    pub static mut OnigDefaultSyntax: *mut OnigSyntaxType;
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigCaptureTreeNodeStruct {
    pub group: ::std::os::raw::c_int,
    pub beg: ::std::os::raw::c_int,
    pub end: ::std::os::raw::c_int,
    pub allocated: ::std::os::raw::c_int,
    pub num_childs: ::std::os::raw::c_int,
    pub childs: *mut *mut OnigCaptureTreeNodeStruct,
}
pub type OnigCaptureTreeNode = OnigCaptureTreeNodeStruct;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct re_registers {
    pub allocated: ::std::os::raw::c_int,
    pub num_regs: ::std::os::raw::c_int,
    pub beg: *mut ::std::os::raw::c_int,
    pub end: *mut ::std::os::raw::c_int,
    pub history_root: *mut OnigCaptureTreeNode,
}
pub type OnigRegion = re_registers;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigErrorInfo {
    pub enc: OnigEncoding,
    pub par: *mut OnigUChar,
    pub par_end: *mut OnigUChar,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigRepeatRange {
    pub lower: ::std::os::raw::c_int,
    pub upper: ::std::os::raw::c_int,
}
pub type OnigWarnFunc =
    ::std::option::Option<unsafe extern "C" fn(s: *const ::std::os::raw::c_char)>;
extern "C" {
    pub fn onig_null_warn(s: *const ::std::os::raw::c_char);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct re_pattern_buffer {
    _unused: [u8; 0],
}
pub type OnigRegexType = re_pattern_buffer;
pub type OnigRegex = *mut OnigRegexType;
pub type regex_t = OnigRegexType;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OnigRegSetStruct {
    _unused: [u8; 0],
}
pub type OnigRegSet = OnigRegSetStruct;
pub const OnigRegSetLead_ONIG_REGSET_POSITION_LEAD: OnigRegSetLead = 0;
pub const OnigRegSetLead_ONIG_REGSET_REGEX_LEAD: OnigRegSetLead = 1;
pub const OnigRegSetLead_ONIG_REGSET_PRIORITY_TO_REGEX_ORDER: OnigRegSetLead = 2;
pub type OnigRegSetLead = u32;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigCompileInfo {
    pub num_of_elements: ::std::os::raw::c_int,
    pub pattern_enc: OnigEncoding,
    pub target_enc: OnigEncoding,
    pub syntax: *mut OnigSyntaxType,
    pub option: OnigOptionType,
    pub case_fold_flag: OnigCaseFoldType,
}
pub const OnigCalloutIn_ONIG_CALLOUT_IN_PROGRESS: OnigCalloutIn = 1;
pub const OnigCalloutIn_ONIG_CALLOUT_IN_RETRACTION: OnigCalloutIn = 2;
pub type OnigCalloutIn = u32;
pub const OnigCalloutOf_ONIG_CALLOUT_OF_CONTENTS: OnigCalloutOf = 0;
pub const OnigCalloutOf_ONIG_CALLOUT_OF_NAME: OnigCalloutOf = 1;
pub type OnigCalloutOf = u32;
pub const OnigCalloutType_ONIG_CALLOUT_TYPE_SINGLE: OnigCalloutType = 0;
pub const OnigCalloutType_ONIG_CALLOUT_TYPE_START_CALL: OnigCalloutType = 1;
pub const OnigCalloutType_ONIG_CALLOUT_TYPE_BOTH_CALL: OnigCalloutType = 2;
pub const OnigCalloutType_ONIG_CALLOUT_TYPE_START_MARK_END_CALL: OnigCalloutType = 3;
pub type OnigCalloutType = u32;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OnigCalloutArgsStruct {
    _unused: [u8; 0],
}
pub type OnigCalloutArgs = OnigCalloutArgsStruct;
pub type OnigCalloutFunc = ::std::option::Option<
    unsafe extern "C" fn(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
pub const OnigCalloutResult_ONIG_CALLOUT_FAIL: OnigCalloutResult = 1;
pub const OnigCalloutResult_ONIG_CALLOUT_SUCCESS: OnigCalloutResult = 0;
pub type OnigCalloutResult = u32;
pub const OnigType_ONIG_TYPE_VOID: OnigType = 0;
pub const OnigType_ONIG_TYPE_LONG: OnigType = 1;
pub const OnigType_ONIG_TYPE_CHAR: OnigType = 2;
pub const OnigType_ONIG_TYPE_STRING: OnigType = 4;
pub const OnigType_ONIG_TYPE_POINTER: OnigType = 8;
pub const OnigType_ONIG_TYPE_TAG: OnigType = 16;
pub type OnigType = u32;
#[repr(C)]
#[derive(Copy, Clone)]
pub union OnigValue {
    pub l: ::std::os::raw::c_long,
    pub c: OnigCodePoint,
    pub s: OnigValue__bindgen_ty_1,
    pub p: *mut ::std::os::raw::c_void,
    pub tag: ::std::os::raw::c_int,
    _bindgen_union_align: [u64; 2usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OnigValue__bindgen_ty_1 {
    pub start: *mut OnigUChar,
    pub end: *mut OnigUChar,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OnigMatchParamStruct {
    _unused: [u8; 0],
}
pub type OnigMatchParam = OnigMatchParamStruct;
extern "C" {
    pub fn onig_initialize(
        encodings: *mut OnigEncoding,
        number_of_encodings: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_init() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_error_code_to_str(
        s: *mut OnigUChar,
        err_code: ::std::os::raw::c_int,
        ...
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_is_error_code_needs_param(code: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_warn_func(f: OnigWarnFunc);
}
extern "C" {
    pub fn onig_set_verb_warn_func(f: OnigWarnFunc);
}
extern "C" {
    pub fn onig_new(
        arg1: *mut OnigRegex,
        pattern: *const OnigUChar,
        pattern_end: *const OnigUChar,
        option: OnigOptionType,
        enc: OnigEncoding,
        syntax: *mut OnigSyntaxType,
        einfo: *mut OnigErrorInfo,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_reg_init(
        reg: OnigRegex,
        option: OnigOptionType,
        case_fold_flag: OnigCaseFoldType,
        enc: OnigEncoding,
        syntax: *mut OnigSyntaxType,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_new_without_alloc(
        arg1: OnigRegex,
        pattern: *const OnigUChar,
        pattern_end: *const OnigUChar,
        option: OnigOptionType,
        enc: OnigEncoding,
        syntax: *mut OnigSyntaxType,
        einfo: *mut OnigErrorInfo,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_new_deluxe(
        reg: *mut OnigRegex,
        pattern: *const OnigUChar,
        pattern_end: *const OnigUChar,
        ci: *mut OnigCompileInfo,
        einfo: *mut OnigErrorInfo,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_free(arg1: OnigRegex);
}
extern "C" {
    pub fn onig_free_body(arg1: OnigRegex);
}
extern "C" {
    pub fn onig_scan(
        reg: OnigRegex,
        str: *const OnigUChar,
        end: *const OnigUChar,
        region: *mut OnigRegion,
        option: OnigOptionType,
        scan_callback: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: ::std::os::raw::c_int,
                arg2: ::std::os::raw::c_int,
                arg3: *mut OnigRegion,
                arg4: *mut ::std::os::raw::c_void,
            ) -> ::std::os::raw::c_int,
        >,
        callback_arg: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_search(
        arg1: OnigRegex,
        str: *const OnigUChar,
        end: *const OnigUChar,
        start: *const OnigUChar,
        range: *const OnigUChar,
        region: *mut OnigRegion,
        option: OnigOptionType,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_search_with_param(
        arg1: OnigRegex,
        str: *const OnigUChar,
        end: *const OnigUChar,
        start: *const OnigUChar,
        range: *const OnigUChar,
        region: *mut OnigRegion,
        option: OnigOptionType,
        mp: *mut OnigMatchParam,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_match(
        arg1: OnigRegex,
        str: *const OnigUChar,
        end: *const OnigUChar,
        at: *const OnigUChar,
        region: *mut OnigRegion,
        option: OnigOptionType,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_match_with_param(
        arg1: OnigRegex,
        str: *const OnigUChar,
        end: *const OnigUChar,
        at: *const OnigUChar,
        region: *mut OnigRegion,
        option: OnigOptionType,
        mp: *mut OnigMatchParam,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_regset_new(
        rset: *mut *mut OnigRegSet,
        n: ::std::os::raw::c_int,
        regs: *mut *mut regex_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_regset_add(set: *mut OnigRegSet, reg: *mut regex_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_regset_replace(
        set: *mut OnigRegSet,
        at: ::std::os::raw::c_int,
        reg: *mut regex_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_regset_free(set: *mut OnigRegSet);
}
extern "C" {
    pub fn onig_regset_number_of_regex(set: *mut OnigRegSet) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_regset_get_regex(set: *mut OnigRegSet, at: ::std::os::raw::c_int) -> *mut regex_t;
}
extern "C" {
    pub fn onig_regset_get_region(
        set: *mut OnigRegSet,
        at: ::std::os::raw::c_int,
    ) -> *mut OnigRegion;
}
extern "C" {
    pub fn onig_regset_search(
        set: *mut OnigRegSet,
        str: *const OnigUChar,
        end: *const OnigUChar,
        start: *const OnigUChar,
        range: *const OnigUChar,
        lead: OnigRegSetLead,
        option: OnigOptionType,
        rmatch_pos: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_regset_search_with_param(
        set: *mut OnigRegSet,
        str: *const OnigUChar,
        end: *const OnigUChar,
        start: *const OnigUChar,
        range: *const OnigUChar,
        lead: OnigRegSetLead,
        option: OnigOptionType,
        mps: *mut *mut OnigMatchParam,
        rmatch_pos: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_region_new() -> *mut OnigRegion;
}
extern "C" {
    pub fn onig_region_init(region: *mut OnigRegion);
}
extern "C" {
    pub fn onig_region_free(region: *mut OnigRegion, free_self: ::std::os::raw::c_int);
}
extern "C" {
    pub fn onig_region_copy(to: *mut OnigRegion, from: *mut OnigRegion);
}
extern "C" {
    pub fn onig_region_clear(region: *mut OnigRegion);
}
extern "C" {
    pub fn onig_region_resize(
        region: *mut OnigRegion,
        n: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_region_set(
        region: *mut OnigRegion,
        at: ::std::os::raw::c_int,
        beg: ::std::os::raw::c_int,
        end: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_name_to_group_numbers(
        reg: OnigRegex,
        name: *const OnigUChar,
        name_end: *const OnigUChar,
        nums: *mut *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_name_to_backref_number(
        reg: OnigRegex,
        name: *const OnigUChar,
        name_end: *const OnigUChar,
        region: *mut OnigRegion,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_foreach_name(
        reg: OnigRegex,
        func: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *const OnigUChar,
                arg2: *const OnigUChar,
                arg3: ::std::os::raw::c_int,
                arg4: *mut ::std::os::raw::c_int,
                arg5: OnigRegex,
                arg6: *mut ::std::os::raw::c_void,
            ) -> ::std::os::raw::c_int,
        >,
        arg: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_number_of_names(reg: OnigRegex) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_number_of_captures(reg: OnigRegex) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_number_of_capture_histories(reg: OnigRegex) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_capture_tree(region: *mut OnigRegion) -> *mut OnigCaptureTreeNode;
}
extern "C" {
    pub fn onig_capture_tree_traverse(
        region: *mut OnigRegion,
        at: ::std::os::raw::c_int,
        callback_func: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: ::std::os::raw::c_int,
                arg2: ::std::os::raw::c_int,
                arg3: ::std::os::raw::c_int,
                arg4: ::std::os::raw::c_int,
                arg5: ::std::os::raw::c_int,
                arg6: *mut ::std::os::raw::c_void,
            ) -> ::std::os::raw::c_int,
        >,
        arg: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_noname_group_capture_is_active(reg: OnigRegex) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_encoding(reg: OnigRegex) -> OnigEncoding;
}
extern "C" {
    pub fn onig_get_options(reg: OnigRegex) -> OnigOptionType;
}
extern "C" {
    pub fn onig_get_case_fold_flag(reg: OnigRegex) -> OnigCaseFoldType;
}
extern "C" {
    pub fn onig_get_syntax(reg: OnigRegex) -> *mut OnigSyntaxType;
}
extern "C" {
    pub fn onig_set_default_syntax(syntax: *mut OnigSyntaxType) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_copy_syntax(to: *mut OnigSyntaxType, from: *mut OnigSyntaxType);
}
extern "C" {
    pub fn onig_get_syntax_op(syntax: *mut OnigSyntaxType) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn onig_get_syntax_op2(syntax: *mut OnigSyntaxType) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn onig_get_syntax_behavior(syntax: *mut OnigSyntaxType) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn onig_get_syntax_options(syntax: *mut OnigSyntaxType) -> OnigOptionType;
}
extern "C" {
    pub fn onig_set_syntax_op(syntax: *mut OnigSyntaxType, op: ::std::os::raw::c_uint);
}
extern "C" {
    pub fn onig_set_syntax_op2(syntax: *mut OnigSyntaxType, op2: ::std::os::raw::c_uint);
}
extern "C" {
    pub fn onig_set_syntax_behavior(syntax: *mut OnigSyntaxType, behavior: ::std::os::raw::c_uint);
}
extern "C" {
    pub fn onig_set_syntax_options(syntax: *mut OnigSyntaxType, options: OnigOptionType);
}
extern "C" {
    pub fn onig_set_meta_char(
        syntax: *mut OnigSyntaxType,
        what: ::std::os::raw::c_uint,
        code: OnigCodePoint,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_copy_encoding(to: OnigEncoding, from: OnigEncoding);
}
extern "C" {
    pub fn onig_get_default_case_fold_flag() -> OnigCaseFoldType;
}
extern "C" {
    pub fn onig_set_default_case_fold_flag(
        case_fold_flag: OnigCaseFoldType,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_match_stack_limit_size() -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn onig_set_match_stack_limit_size(size: ::std::os::raw::c_uint) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_retry_limit_in_match() -> ::std::os::raw::c_ulong;
}
extern "C" {
    pub fn onig_set_retry_limit_in_match(n: ::std::os::raw::c_ulong) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_retry_limit_in_search() -> ::std::os::raw::c_ulong;
}
extern "C" {
    pub fn onig_set_retry_limit_in_search(n: ::std::os::raw::c_ulong) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_parse_depth_limit() -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn onig_set_capture_num_limit(num: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_parse_depth_limit(depth: ::std::os::raw::c_uint) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_subexp_call_max_nest_level() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_subexp_call_max_nest_level(
        level: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_unicode_define_user_property(
        name: *const ::std::os::raw::c_char,
        ranges: *mut OnigCodePoint,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_end() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_version() -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn onig_copyright() -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn onig_new_match_param() -> *mut OnigMatchParam;
}
extern "C" {
    pub fn onig_free_match_param(p: *mut OnigMatchParam);
}
extern "C" {
    pub fn onig_free_match_param_content(p: *mut OnigMatchParam);
}
extern "C" {
    pub fn onig_initialize_match_param(mp: *mut OnigMatchParam) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_match_stack_limit_size_of_match_param(
        param: *mut OnigMatchParam,
        limit: ::std::os::raw::c_uint,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_retry_limit_in_match_of_match_param(
        param: *mut OnigMatchParam,
        limit: ::std::os::raw::c_ulong,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_retry_limit_in_search_of_match_param(
        param: *mut OnigMatchParam,
        limit: ::std::os::raw::c_ulong,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_progress_callout_of_match_param(
        param: *mut OnigMatchParam,
        f: OnigCalloutFunc,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_retraction_callout_of_match_param(
        param: *mut OnigMatchParam,
        f: OnigCalloutFunc,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_callout_user_data_of_match_param(
        param: *mut OnigMatchParam,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_progress_callout() -> OnigCalloutFunc;
}
extern "C" {
    pub fn onig_set_progress_callout(f: OnigCalloutFunc) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_retraction_callout() -> OnigCalloutFunc;
}
extern "C" {
    pub fn onig_set_retraction_callout(f: OnigCalloutFunc) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_callout_of_name(
        enc: OnigEncoding,
        type_: OnigCalloutType,
        name: *mut OnigUChar,
        name_end: *mut OnigUChar,
        callout_in: ::std::os::raw::c_int,
        callout: OnigCalloutFunc,
        end_callout: OnigCalloutFunc,
        arg_num: ::std::os::raw::c_int,
        arg_types: *mut ::std::os::raw::c_uint,
        optional_arg_num: ::std::os::raw::c_int,
        opt_defaults: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_name_by_name_id(id: ::std::os::raw::c_int) -> *mut OnigUChar;
}
extern "C" {
    pub fn onig_get_callout_num_by_tag(
        reg: OnigRegex,
        tag: *const OnigUChar,
        tag_end: *const OnigUChar,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_data_by_tag(
        reg: OnigRegex,
        mp: *mut OnigMatchParam,
        tag: *const OnigUChar,
        tag_end: *const OnigUChar,
        slot: ::std::os::raw::c_int,
        type_: *mut OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_callout_data_by_tag(
        reg: OnigRegex,
        mp: *mut OnigMatchParam,
        tag: *const OnigUChar,
        tag_end: *const OnigUChar,
        slot: ::std::os::raw::c_int,
        type_: OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_num_by_callout_args(
        args: *mut OnigCalloutArgs,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_in_by_callout_args(args: *mut OnigCalloutArgs) -> OnigCalloutIn;
}
extern "C" {
    pub fn onig_get_name_id_by_callout_args(args: *mut OnigCalloutArgs) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_contents_by_callout_args(args: *mut OnigCalloutArgs) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_contents_end_by_callout_args(args: *mut OnigCalloutArgs) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_args_num_by_callout_args(args: *mut OnigCalloutArgs) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_passed_args_num_by_callout_args(
        args: *mut OnigCalloutArgs,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_arg_by_callout_args(
        args: *mut OnigCalloutArgs,
        index: ::std::os::raw::c_int,
        type_: *mut OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_string_by_callout_args(args: *mut OnigCalloutArgs) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_string_end_by_callout_args(args: *mut OnigCalloutArgs) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_start_by_callout_args(args: *mut OnigCalloutArgs) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_right_range_by_callout_args(args: *mut OnigCalloutArgs) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_current_by_callout_args(args: *mut OnigCalloutArgs) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_regex_by_callout_args(args: *mut OnigCalloutArgs) -> OnigRegex;
}
extern "C" {
    pub fn onig_get_retry_counter_by_callout_args(
        args: *mut OnigCalloutArgs,
    ) -> ::std::os::raw::c_ulong;
}
extern "C" {
    pub fn onig_callout_tag_is_exist_at_callout_num(
        reg: OnigRegex,
        callout_num: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_tag_start(
        reg: OnigRegex,
        callout_num: ::std::os::raw::c_int,
    ) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_callout_tag_end(
        reg: OnigRegex,
        callout_num: ::std::os::raw::c_int,
    ) -> *const OnigUChar;
}
extern "C" {
    pub fn onig_get_callout_data_dont_clear_old(
        reg: OnigRegex,
        mp: *mut OnigMatchParam,
        callout_num: ::std::os::raw::c_int,
        slot: ::std::os::raw::c_int,
        type_: *mut OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_data_by_callout_args_self_dont_clear_old(
        args: *mut OnigCalloutArgs,
        slot: ::std::os::raw::c_int,
        type_: *mut OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_data(
        reg: OnigRegex,
        mp: *mut OnigMatchParam,
        callout_num: ::std::os::raw::c_int,
        slot: ::std::os::raw::c_int,
        type_: *mut OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_data_by_callout_args(
        args: *mut OnigCalloutArgs,
        callout_num: ::std::os::raw::c_int,
        slot: ::std::os::raw::c_int,
        type_: *mut OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_callout_data_by_callout_args_self(
        args: *mut OnigCalloutArgs,
        slot: ::std::os::raw::c_int,
        type_: *mut OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_callout_data(
        reg: OnigRegex,
        mp: *mut OnigMatchParam,
        callout_num: ::std::os::raw::c_int,
        slot: ::std::os::raw::c_int,
        type_: OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_callout_data_by_callout_args(
        args: *mut OnigCalloutArgs,
        callout_num: ::std::os::raw::c_int,
        slot: ::std::os::raw::c_int,
        type_: OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_set_callout_data_by_callout_args_self(
        args: *mut OnigCalloutArgs,
        slot: ::std::os::raw::c_int,
        type_: OnigType,
        val: *mut OnigValue,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_capture_range_in_callout(
        args: *mut OnigCalloutArgs,
        mem_num: ::std::os::raw::c_int,
        begin: *mut ::std::os::raw::c_int,
        end: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_get_used_stack_size_in_callout(
        args: *mut OnigCalloutArgs,
        used_num: *mut ::std::os::raw::c_int,
        used_bytes: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_builtin_fail(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_builtin_mismatch(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_builtin_error(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_builtin_count(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_builtin_total_count(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_builtin_max(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_builtin_cmp(
        args: *mut OnigCalloutArgs,
        user_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn onig_setup_builtin_monitors_by_ascii_encoded_name(
        fp: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
