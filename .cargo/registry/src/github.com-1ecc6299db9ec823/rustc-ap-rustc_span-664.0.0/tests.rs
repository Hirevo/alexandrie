use super::*;

#[test]
fn test_lookup_line() {
    let lines = &[BytePos(3), BytePos(17), BytePos(28)];

    assert_eq!(lookup_line(lines, BytePos(0)), -1);
    assert_eq!(lookup_line(lines, BytePos(3)), 0);
    assert_eq!(lookup_line(lines, BytePos(4)), 0);

    assert_eq!(lookup_line(lines, BytePos(16)), 0);
    assert_eq!(lookup_line(lines, BytePos(17)), 1);
    assert_eq!(lookup_line(lines, BytePos(18)), 1);

    assert_eq!(lookup_line(lines, BytePos(28)), 2);
    assert_eq!(lookup_line(lines, BytePos(29)), 2);
}

#[test]
fn test_normalize_newlines() {
    fn check(before: &str, after: &str, expected_positions: &[u32]) {
        let mut actual = before.to_string();
        let mut actual_positions = vec![];
        normalize_newlines(&mut actual, &mut actual_positions);
        let actual_positions: Vec<_> = actual_positions.into_iter().map(|nc| nc.pos.0).collect();
        assert_eq!(actual.as_str(), after);
        assert_eq!(actual_positions, expected_positions);
    }
    check("", "", &[]);
    check("\n", "\n", &[]);
    check("\r", "\r", &[]);
    check("\r\r", "\r\r", &[]);
    check("\r\n", "\n", &[1]);
    check("hello world", "hello world", &[]);
    check("hello\nworld", "hello\nworld", &[]);
    check("hello\r\nworld", "hello\nworld", &[6]);
    check("\r\nhello\r\nworld\r\n", "\nhello\nworld\n", &[1, 7, 13]);
    check("\r\r\n", "\r\n", &[2]);
    check("hello\rworld", "hello\rworld", &[]);
}
