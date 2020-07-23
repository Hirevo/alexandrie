use std::fmt;

use super::{Mime, ParamKind, ParamName, ParamValue};

/// Parse a string into a mime type.
/// Follows the [WHATWG MIME parsing algorithm](https://mimesniff.spec.whatwg.org/#parsing-a-mime-type)
pub(crate) fn parse(input: &str) -> crate::Result<Mime> {
    // 1
    let input = input.trim_matches(is_http_whitespace_char);

    // 3.
    let (basetype, input) = collect_code_point_sequence_char(input, '/');

    // 4.
    crate::ensure!(!basetype.is_empty(), "MIME type should not be empty");
    crate::ensure!(
        basetype.chars().all(is_http_token_code_point),
        "MIME type should ony contain valid HTTP token code points"
    );

    // 5.
    crate::ensure!(!input.is_empty(), "MIME must contain a sub type");

    // 6.
    let input = &input[1..];

    // 7.
    let (subtype, input) = collect_code_point_sequence_char(input, ';');

    // 8.
    let subtype = subtype.trim_end_matches(is_http_whitespace_char);

    // 9.
    crate::ensure!(!subtype.is_empty(), "MIME sub type should not be empty");
    crate::ensure!(
        subtype.chars().all(is_http_token_code_point),
        "MIME sub type should ony contain valid HTTP token code points"
    );

    // 10.
    let basetype = basetype.to_ascii_lowercase();
    let subtype = subtype.to_ascii_lowercase();
    let mut params = None;

    // 11.
    let mut input = input;
    while !input.is_empty() {
        // 1.
        input = &input[1..];

        // 2.
        input = input.trim_start_matches(is_http_whitespace_char);

        // 3.
        let (parameter_name, new_input) =
            collect_code_point_sequence_slice(input, &[';', '='] as &[char]);
        input = new_input;

        // 4.
        let parameter_name = parameter_name.to_ascii_lowercase();

        if input.is_empty() {
            // 6.
            break;
        } else {
            // 5.
            if input.starts_with(';') {
                continue;
            } else {
                // It's a '='
                input = &input[1..];
            }
        }

        let parameter_value = if input.starts_with('"') {
            // 8.
            // implementation of https://fetch.spec.whatwg.org/#collect-an-http-quoted-string
            let (parameter_value, new_input) = collect_http_quoted_string(input);
            let (_, new_input) = collect_code_point_sequence_char(new_input, ';');
            input = new_input;
            parameter_value
        } else {
            // 9.
            let (parameter_value, new_input) = collect_code_point_sequence_char(input, ';');
            input = new_input;
            let parameter_value = parameter_value.trim_end_matches(is_http_whitespace_char);
            if parameter_value.is_empty() {
                continue;
            }
            parameter_value.to_owned()
        };

        // 10.
        if !parameter_name.is_empty()
            && parameter_name.chars().all(is_http_token_code_point)
            && parameter_value
                .chars()
                .all(is_http_quoted_string_token_code_point)
        {
            let params = params.get_or_insert_with(Vec::new);
            let name = ParamName(parameter_name.into());
            let value = ParamValue(parameter_value.into());
            if !params.iter().any(|(k, _)| k == &name) {
                params.push((name, value));
            }
        }
    }

    Ok(Mime {
        essence: format!("{}/{}", &basetype, &subtype),
        basetype,
        subtype,
        params: params.map(ParamKind::Vec),
        static_essence: None,
        static_basetype: None,
        static_subtype: None,
    })
}

/// Validates [HTTP token code points](https://mimesniff.spec.whatwg.org/#http-token-code-point)
fn is_http_token_code_point(c: char) -> bool {
    match c {
        '!'
        | '#'
        | '$'
        | '%'
        | '&'
        | '\''
        | '*'
        | '+'
        | '-'
        | '.'
        | '^'
        | '_'
        | '`'
        | '|'
        | '~'
        | 'a'..='z'
        | 'A'..='Z'
        | '0'..='9' => true,
        _ => false,
    }
}

/// Validates [HTTP quoted-string token code points](https://mimesniff.spec.whatwg.org/#http-quoted-string-token-code-point)
fn is_http_quoted_string_token_code_point(c: char) -> bool {
    match c {
        '\t' | ' '..='~' | '\u{80}'..='\u{FF}' => true,
        _ => false,
    }
}

/// Is a [HTTP whitespace](https://fetch.spec.whatwg.org/#http-whitespace)
fn is_http_whitespace_char(c: char) -> bool {
    match c {
        '\n' | '\r' | '\t' | ' ' => true,
        _ => false,
    }
}

/// [code point sequence collection](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points)
fn collect_code_point_sequence_char(input: &str, delimiter: char) -> (&str, &str) {
    input.split_at(input.find(delimiter).unwrap_or_else(|| input.len()))
}

/// [code point sequence collection](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points)
fn collect_code_point_sequence_slice<'a>(input: &'a str, delimiter: &[char]) -> (&'a str, &'a str) {
    input.split_at(input.find(delimiter).unwrap_or_else(|| input.len()))
}

/// [HTTP quoted string collection](https://fetch.spec.whatwg.org/#collect-an-http-quoted-string)
///
/// Assumes that the first char is '"'
fn collect_http_quoted_string(mut input: &str) -> (String, &str) {
    // 2.
    let mut value = String::new();
    // 4.
    input = &input[1..];
    // 5.
    loop {
        // 1.
        let (add_value, new_input) =
            collect_code_point_sequence_slice(input, &['"', '\\'] as &[char]);
        value.push_str(add_value);
        let mut chars = new_input.chars();
        // 3.
        if let Some(quote_or_backslash) = chars.next() {
            // 4.
            input = chars.as_str();
            //5.
            if quote_or_backslash == '\\' {
                if let Some(c) = chars.next() {
                    // 2.
                    value.push(c);
                    // 3.
                    input = chars.as_str();
                } else {
                    // 1.
                    value.push('\\');
                    break;
                }
            } else {
                // 6.
                break;
            }
        } else {
            // 2
            break;
        }
    }
    (value, input)
}

/// Implementation of [WHATWG MIME serialization algorithm](https://mimesniff.spec.whatwg.org/#serializing-a-mime-type)
pub(crate) fn format(mime_type: &Mime, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(essence) = mime_type.static_essence {
        write!(f, "{}", essence)?
    } else {
        write!(f, "{}", &mime_type.essence)?
    }
    if let Some(params) = &mime_type.params {
        match params {
            ParamKind::Utf8 => write!(f, ";charset=utf-8")?,
            ParamKind::Vec(params) => {
                for (name, value) in params {
                    if value.0.chars().all(is_http_token_code_point) && !value.0.is_empty() {
                        write!(f, ";{}={}", name, value)?;
                    } else {
                        write!(
                            f,
                            ";{}=\"{}\"",
                            name,
                            value
                                .0
                                .chars()
                                .flat_map(|c| match c {
                                    '"' | '\\' => EscapeMimeValue {
                                        state: EscapeMimeValueState::Backslash(c)
                                    },
                                    c => EscapeMimeValue {
                                        state: EscapeMimeValueState::Char(c)
                                    },
                                })
                                .collect::<String>()
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

struct EscapeMimeValue {
    state: EscapeMimeValueState,
}

#[derive(Clone, Debug)]
enum EscapeMimeValueState {
    Done,
    Char(char),
    Backslash(char),
}

impl Iterator for EscapeMimeValue {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.state {
            EscapeMimeValueState::Done => None,
            EscapeMimeValueState::Char(c) => {
                self.state = EscapeMimeValueState::Done;
                Some(c)
            }
            EscapeMimeValueState::Backslash(c) => {
                self.state = EscapeMimeValueState::Char(c);
                Some('\\')
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.state {
            EscapeMimeValueState::Done => (0, Some(0)),
            EscapeMimeValueState::Char(_) => (1, Some(1)),
            EscapeMimeValueState::Backslash(_) => (2, Some(2)),
        }
    }
}

#[test]
fn test() {
    let mime = parse("text/html").unwrap();
    assert_eq!(mime.basetype(), "text");
    assert_eq!(mime.subtype(), "html");

    // technically invalid mime, but allow anyway
    let mime = parse("text/html;").unwrap();
    assert_eq!(mime.basetype(), "text");
    assert_eq!(mime.subtype(), "html");

    let mime = parse("text/html; charset=utf-8").unwrap();
    assert_eq!(mime.basetype(), "text");
    assert_eq!(mime.subtype(), "html");
    assert_eq!(mime.param("charset").unwrap(), "utf-8");

    let mime = parse("text/html; charset=utf-8;").unwrap();
    assert_eq!(mime.basetype(), "text");
    assert_eq!(mime.subtype(), "html");
    assert_eq!(mime.param("charset").unwrap(), "utf-8");

    assert!(parse("text").is_err());
    assert!(parse("text/").is_err());
    assert!(parse("t/").is_err());
    assert!(parse("t/h").is_ok());
}

/// Web Platform tests for MIME type parsing
/// From https://github.com/web-platform-tests/wpt/blob/master/mimesniff/mime-types/resources/mime-types.json
#[test]
fn whatwag_tests() {
    fn assert_parse(input: &str, expected: &str) {
        let actual = parse(input).unwrap();
        assert_eq!(actual.to_string(), expected);
    }

    fn assert_fails(input: &str) {
        assert!(parse(input).is_err());
    }

    fn assert_parse_and_encoding(
        input: &str,
        expected: &str,
        _encoding: impl Into<Option<&'static str>>,
    ) {
        //TODO: check encoding
        assert_parse(input, expected);
    }

    // Basics
    assert_parse_and_encoding("text/html;charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding("TEXT/HTML;CHARSET=GBK", "text/html;charset=GBK", "GBK");

    //" Legacy comment syntax"
    assert_parse_and_encoding("text/html;charset=gbk(", "text/html;charset=\"gbk(\"", None);
    assert_parse_and_encoding(
        "text/html;x=(;charset=gbk",
        "text/html;x=\"(\";charset=gbk",
        "GBK",
    );

    // Duplicate parameter
    assert_parse_and_encoding(
        "text/html;charset=gbk;charset=windows-1255",
        "text/html;charset=gbk",
        "GBK",
    );
    assert_parse_and_encoding(
        "text/html;charset=();charset=GBK",
        "text/html;charset=\"()\"",
        None,
    );

    // Spaces
    assert_parse_and_encoding("text/html;charset =gbk", "text/html", None);
    assert_parse_and_encoding("text/html ;charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding("text/html; charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding(
        "text/html;charset= gbk",
        "text/html;charset=\" gbk\"",
        "GBK",
    );
    assert_parse_and_encoding(
        "text/html;charset= \"gbk\"",
        "text/html;charset=\" \\\"gbk\\\"\"",
        None,
    );

    // 0x0B and 0x0C
    assert_parse_and_encoding("text/html;charset=\u{000B}gbk", "text/html", None);
    assert_parse_and_encoding("text/html;charset=\u{000C}gbk", "text/html", None);
    assert_parse_and_encoding("text/html;\u{000B}charset=gbk", "text/html", None);
    assert_parse_and_encoding("text/html;\u{000C}charset=gbk", "text/html", None);

    // Single quotes are a token, not a delimiter
    assert_parse_and_encoding("text/html;charset='gbk'", "text/html;charset='gbk'", None);
    assert_parse_and_encoding("text/html;charset='gbk", "text/html;charset='gbk", None);
    assert_parse_and_encoding("text/html;charset=gbk'", "text/html;charset=gbk'", None);
    assert_parse_and_encoding(
        "text/html;charset=';charset=GBK",
        "text/html;charset='",
        None,
    );

    // Invalid parameters
    assert_parse_and_encoding("text/html;test;charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding(
        "text/html;test=;charset=gbk",
        "text/html;charset=gbk",
        "GBK",
    );
    assert_parse_and_encoding("text/html;';charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding("text/html;\";charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding("text/html ; ; charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding("text/html;;;;charset=gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding(
        "text/html;charset= \"\u{007F};charset=GBK",
        "text/html;charset=GBK",
        "GBK",
    );
    assert_parse_and_encoding(
        "text/html;charset=\"\u{007F};charset=foo\";charset=GBK",
        "text/html;charset=GBK",
        "GBK",
    );

    // Double quotes"
    assert_parse_and_encoding("text/html;charset=\"gbk\"", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding("text/html;charset=\"gbk", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding(
        "text/html;charset=gbk\"",
        "text/html;charset=\"gbk\\\"\"",
        None,
    );
    assert_parse_and_encoding(
        "text/html;charset=\" gbk\"",
        "text/html;charset=\" gbk\"",
        "GBK",
    );
    assert_parse_and_encoding(
        "text/html;charset=\"gbk \"",
        "text/html;charset=\"gbk \"",
        "GBK",
    );
    assert_parse_and_encoding(
        "text/html;charset=\"\\ gbk\"",
        "text/html;charset=\" gbk\"",
        "GBK",
    );
    assert_parse_and_encoding(
        "text/html;charset=\"\\g\\b\\k\"",
        "text/html;charset=gbk",
        "GBK",
    );
    assert_parse_and_encoding("text/html;charset=\"gbk\"x", "text/html;charset=gbk", "GBK");
    assert_parse_and_encoding(
        "text/html;charset=\"\";charset=GBK",
        "text/html;charset=\"\"",
        None,
    );
    assert_parse_and_encoding(
        "text/html;charset=\";charset=GBK",
        "text/html;charset=\";charset=GBK\"",
        None,
    );

    // Unexpected code points
    assert_parse_and_encoding(
        "text/html;charset={gbk}",
        "text/html;charset=\"{gbk}\"",
        None,
    );

    // Parameter name longer than 127
    assert_parse_and_encoding("text/html;0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789=x;charset=gbk", "text/html;0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789=x;charset=gbk", "GBK");

    // type/subtype longer than 127
    assert_parse("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789/0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789", "0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789/0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789");

    // Valid
    assert_parse("!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz/!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz;!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz=!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz", "!#$%&'*+-.^_`|~0123456789abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz/!#$%&'*+-.^_`|~0123456789abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz;!#$%&'*+-.^_`|~0123456789abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz=!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");
    assert_parse("x/x;x=\"\t !\\\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\u{0080}\u{0081}\u{0082}\u{0083}\u{0084}\u{0085}\u{0086}\u{0087}\u{0088}\u{0089}\u{008A}\u{008B}\u{008C}\u{008D}\u{008E}\u{008F}\u{0090}\u{0091}\u{0092}\u{0093}\u{0094}\u{0095}\u{0096}\u{0097}\u{0098}\u{0099}\u{009A}\u{009B}\u{009C}\u{009D}\u{009E}\u{009F}\u{00A0}\u{00A1}\u{00A2}\u{00A3}\u{00A4}\u{00A5}\u{00A6}\u{00A7}\u{00A8}\u{00A9}\u{00AA}\u{00AB}\u{00AC}\u{00AD}\u{00AE}\u{00AF}\u{00B0}\u{00B1}\u{00B2}\u{00B3}\u{00B4}\u{00B5}\u{00B6}\u{00B7}\u{00B8}\u{00B9}\u{00BA}\u{00BB}\u{00BC}\u{00BD}\u{00BE}\u{00BF}\u{00C0}\u{00C1}\u{00C2}\u{00C3}\u{00C4}\u{00C5}\u{00C6}\u{00C7}\u{00C8}\u{00C9}\u{00CA}\u{00CB}\u{00CC}\u{00CD}\u{00CE}\u{00CF}\u{00D0}\u{00D1}\u{00D2}\u{00D3}\u{00D4}\u{00D5}\u{00D6}\u{00D7}\u{00D8}\u{00D9}\u{00DA}\u{00DB}\u{00DC}\u{00DD}\u{00DE}\u{00DF}\u{00E0}\u{00E1}\u{00E2}\u{00E3}\u{00E4}\u{00E5}\u{00E6}\u{00E7}\u{00E8}\u{00E9}\u{00EA}\u{00EB}\u{00EC}\u{00ED}\u{00EE}\u{00EF}\u{00F0}\u{00F1}\u{00F2}\u{00F3}\u{00F4}\u{00F5}\u{00F6}\u{00F7}\u{00F8}\u{00F9}\u{00FA}\u{00FB}\u{00FC}\u{00FD}\u{00FE}\u{00FF}\"", "x/x;x=\"\t !\\\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\u{0080}\u{0081}\u{0082}\u{0083}\u{0084}\u{0085}\u{0086}\u{0087}\u{0088}\u{0089}\u{008A}\u{008B}\u{008C}\u{008D}\u{008E}\u{008F}\u{0090}\u{0091}\u{0092}\u{0093}\u{0094}\u{0095}\u{0096}\u{0097}\u{0098}\u{0099}\u{009A}\u{009B}\u{009C}\u{009D}\u{009E}\u{009F}\u{00A0}\u{00A1}\u{00A2}\u{00A3}\u{00A4}\u{00A5}\u{00A6}\u{00A7}\u{00A8}\u{00A9}\u{00AA}\u{00AB}\u{00AC}\u{00AD}\u{00AE}\u{00AF}\u{00B0}\u{00B1}\u{00B2}\u{00B3}\u{00B4}\u{00B5}\u{00B6}\u{00B7}\u{00B8}\u{00B9}\u{00BA}\u{00BB}\u{00BC}\u{00BD}\u{00BE}\u{00BF}\u{00C0}\u{00C1}\u{00C2}\u{00C3}\u{00C4}\u{00C5}\u{00C6}\u{00C7}\u{00C8}\u{00C9}\u{00CA}\u{00CB}\u{00CC}\u{00CD}\u{00CE}\u{00CF}\u{00D0}\u{00D1}\u{00D2}\u{00D3}\u{00D4}\u{00D5}\u{00D6}\u{00D7}\u{00D8}\u{00D9}\u{00DA}\u{00DB}\u{00DC}\u{00DD}\u{00DE}\u{00DF}\u{00E0}\u{00E1}\u{00E2}\u{00E3}\u{00E4}\u{00E5}\u{00E6}\u{00E7}\u{00E8}\u{00E9}\u{00EA}\u{00EB}\u{00EC}\u{00ED}\u{00EE}\u{00EF}\u{00F0}\u{00F1}\u{00F2}\u{00F3}\u{00F4}\u{00F5}\u{00F6}\u{00F7}\u{00F8}\u{00F9}\u{00FA}\u{00FB}\u{00FC}\u{00FD}\u{00FE}\u{00FF}\"");

    // End-of-file handling
    assert_parse("x/x;test", "x/x");
    assert_parse("x/x;test=\"\\", "x/x;test=\"\\\\\"");

    // Whitespace (not handled by generated-mime-types.json or above)
    assert_parse("x/x;x= ", "x/x");
    assert_parse("x/x;x=\t", "x/x");
    assert_parse("x/x\n\r\t ;x=x", "x/x;x=x");
    assert_parse("\n\r\t x/x;x=x\n\r\t ", "x/x;x=x");
    assert_parse("x/x;\n\r\t x=x\n\r\t ;x=y", "x/x;x=x");

    // Latin1
    assert_parse_and_encoding(
        "text/html;test=\u{00FF};charset=gbk",
        "text/html;test=\"\u{00FF}\";charset=gbk",
        "GBK",
    );

    // >Latin1
    assert_parse("x/x;test=\u{FFFD};x=x", "x/x;x=x");

    // Failure
    assert_fails("\u{000B}x/x");
    assert_fails("\u{000C}x/x");
    assert_fails("x/x\u{000B}");
    assert_fails("x/x\u{000C}");
    assert_fails("");
    assert_fails("\t");
    assert_fails("/");
    assert_fails("bogus");
    assert_fails("bogus/");
    assert_fails("bogus/ ");
    assert_fails("bogus/bogus/;");
    assert_fails("</>");
    assert_fails("(/)");
    assert_fails("ÿ/ÿ");
    assert_fails("text/html(;doesnot=matter");
    assert_fails("{/}");
    assert_fails("\u{0100}/\u{0100}");
    assert_fails("text /html");
    assert_fails("text/ html");
    assert_fails("\"text/html\"");
}
