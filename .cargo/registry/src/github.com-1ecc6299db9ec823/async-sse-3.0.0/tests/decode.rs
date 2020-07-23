use async_sse::{decode, Event};
use async_std::io::Cursor;
use async_std::prelude::*;
use std::time::Duration;

/// Assert a Message.
fn assert_message(event: &Event, name: &str, data: &str, id: Option<&'static str>) {
    assert!(event.is_message());
    if let Event::Message(msg) = event {
        assert_eq!(msg.id(), &id.map(|s| s.to_owned()));
        assert_eq!(msg.name(), name);
        assert_eq!(
            String::from_utf8(msg.data().to_owned()).unwrap(),
            String::from_utf8(data.as_bytes().to_owned()).unwrap()
        );
    }
}

/// Assert a Message.
fn assert_retry(event: &Event, dur: u64) {
    assert!(event.is_retry());
    let expected = Duration::from_secs_f64(dur as f64);
    if let Event::Retry(dur) = event {
        assert_eq!(dur, &expected);
    }
}

#[async_std::test]
async fn simple_event() -> http_types::Result<()> {
    let input = Cursor::new("event: add\ndata: test\ndata: test2\n\n");
    let mut reader = decode(input);
    let event = reader.next().await.unwrap()?;
    assert_message(&event, "add", "test\ntest2", None);
    Ok(())
}

#[async_std::test]
async fn decode_stream_when_fed_by_line() -> http_types::Result<()> {
    let reader = decode(Cursor::new(":ok\nevent:message\nid:id1\ndata:data1\n\n"));
    let res = reader.map(|i| i.unwrap()).collect::<Vec<_>>().await;
    assert_eq!(res.len(), 1);
    assert_message(res.get(0).unwrap(), "message", "data1", Some("id1"));
    Ok(())
}

#[async_std::test]
async fn maintain_id_state() -> http_types::Result<()> {
    let reader = decode(Cursor::new("id:1\ndata:messageone\n\ndata:messagetwo\n\n"));
    let mut res = reader.map(|i| i.unwrap()).collect::<Vec<_>>().await;
    assert_eq!(res.len(), 2);
    assert_message(&res.remove(0), "message", "messageone", Some("1"));
    assert_message(&res.remove(0), "message", "messagetwo", Some("1"));
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/event-data.html
#[async_std::test]
async fn event_data() -> http_types::Result<()> {
    femme::with_level(log::LevelFilter::Trace);
    let input = concat!(
        "data:event\n",
        "data:event\n\n",
        ":\n",
        "falsefield:event\n\n",
        "falsefield:event\n",
        "Data:data\n\n",
        "data\n\n",
        "data:end\n\n",
    );
    let mut reader = decode(Cursor::new(input));
    assert_message(
        &reader.next().await.unwrap()?,
        "message",
        "event\nevent",
        None,
    );
    assert_message(&reader.next().await.unwrap()?, "message", "", None);
    assert_message(&reader.next().await.unwrap()?, "message", "end", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-bom.htm
/// The byte order marker should only be stripped at the very start.
#[async_std::test]
async fn bom() -> http_types::Result<()> {
    let mut input = vec![];
    input.extend(b"\xEF\xBB\xBF");
    input.extend(b"data:1\n");
    input.extend(b"\n");
    input.extend(b"\xEF\xBB\xBF");
    input.extend(b"data:2\n");
    input.extend(b"\n");
    input.extend(b"data:3\n");
    input.extend(b"\n");
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "message", "1", None);
    assert_message(&reader.next().await.unwrap()?, "message", "3", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-bom-2.htm
/// Only _one_ byte order marker should be stripped. This has two, which means one will remain
/// in the first line, therefore making the first `data:1` invalid.
#[async_std::test]
async fn bom2() -> http_types::Result<()> {
    let mut input = vec![];
    input.extend(b"\xEF\xBB\xBF");
    input.extend(b"\xEF\xBB\xBF");
    input.extend(b"data:1\n");
    input.extend(b"\n");
    input.extend(b"data:2\n");
    input.extend(b"\n");
    input.extend(b"data:3\n");
    input.extend(b"\n");
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "message", "2", None);
    assert_message(&reader.next().await.unwrap()?, "message", "3", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-comments.htm
#[async_std::test]
async fn comments() -> http_types::Result<()> {
    let longstring = "x".repeat(2049);
    let mut input = concat!("data:1\r", ":\0\n", ":\r\n", "data:2\n", ":").to_string();
    input.push_str(&longstring);
    input.push_str("\r");
    input.push_str("data:3\n");
    input.push_str(":data:fail\r");
    input.push_str(":");
    input.push_str(&longstring);
    input.push_str("\n");
    input.push_str("data:4\n\n");
    let mut reader = decode(Cursor::new(input));
    assert_message(
        &reader.next().await.unwrap()?,
        "message",
        "1\n2\n3\n4",
        None,
    );
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-data-before-final-empty-line.htm
#[async_std::test]
async fn data_before_final_empty_line() -> http_types::Result<()> {
    let input = "retry:1000\ndata:test1\n\nid:test\ndata:test2";
    let mut reader = decode(Cursor::new(input));
    assert_retry(&reader.next().await.unwrap()?, 1000);
    assert_message(&reader.next().await.unwrap()?, "message", "test1", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-data.htm
#[async_std::test]
async fn field_data() -> http_types::Result<()> {
    let input = "data:\n\ndata\ndata\n\ndata:test\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "message", "", None);
    assert_message(&reader.next().await.unwrap()?, "message", "\n", None); // No `:`, so it's empty data + newline.
    assert_message(&reader.next().await.unwrap()?, "message", "test", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-event-empty.htm
#[async_std::test]
async fn field_event_empty() -> http_types::Result<()> {
    let input = "event: \ndata:data\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "", "data", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-event.htm
#[async_std::test]
async fn field_event() -> http_types::Result<()> {
    let input = "event:test\ndata:x\n\ndata:x\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "test", "x", None);
    assert_message(&reader.next().await.unwrap()?, "message", "x", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-id.htm
#[test]
#[ignore]
fn field_id() {
    unimplemented!()
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-id-2.htm
#[test]
#[ignore]
fn field_id_2() {
    unimplemented!()
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-parsing.htm
#[async_std::test]
async fn field_parsing() -> http_types::Result<()> {
    let input = "data:\0\ndata:  2\rData:1\ndata\0:2\ndata:1\r\0data:4\nda-ta:3\rdata_5\ndata:3\rdata:\r\n data:32\ndata:4\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(
        &reader.next().await.unwrap()?,
        "message",
        "\0\n 2\n1\n3\n\n4",
        None,
    );
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-retry-bogus.htm
#[async_std::test]
async fn field_retry_bogus() -> http_types::Result<()> {
    let input = "retry:3000\nretry:1000x\ndata:x\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_retry(&reader.next().await.unwrap()?, 3000);
    assert_message(&reader.next().await.unwrap()?, "message", "x", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-retry-empty.htm
#[async_std::test]
async fn field_retry_empty() -> http_types::Result<()> {
    let input = "retry\ndata:test\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "message", "test", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-retry.htm
#[async_std::test]
async fn field_retry() -> http_types::Result<()> {
    let input = "retry:03000\ndata:x\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_retry(&reader.next().await.unwrap()?, 3000);
    assert_message(&reader.next().await.unwrap()?, "message", "x", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-field-unknown.htm
#[async_std::test]
async fn field_unknown() -> http_types::Result<()> {
    let input =
        "data:test\n data\ndata\nfoobar:xxx\njustsometext\n:thisisacommentyay\ndata:test\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(
        &reader.next().await.unwrap()?,
        "message",
        "test\n\ntest",
        None,
    );
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-leading-space.htm
#[async_std::test]
async fn leading_space() -> http_types::Result<()> {
    let input = "data:\ttest\rdata: \ndata:test\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(
        &reader.next().await.unwrap()?,
        "message",
        "\ttest\n\ntest",
        None,
    );
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-newlines.htm
#[async_std::test]
async fn newlines() -> http_types::Result<()> {
    let input = "data:test\r\ndata\ndata:test\r\n\r";
    let mut reader = decode(Cursor::new(input));
    assert_message(
        &reader.next().await.unwrap()?,
        "message",
        "test\n\ntest",
        None,
    );
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-null-character.html
#[async_std::test]
async fn null_character() -> http_types::Result<()> {
    let input = "data:\0\n\n\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "message", "\0", None);
    assert!(reader.next().await.is_none());
    Ok(())
}

/// https://github.com/web-platform-tests/wpt/blob/master/eventsource/format-utf-8.htm
#[async_std::test]
async fn utf_8() -> http_types::Result<()> {
    let input = b"data:ok\xE2\x80\xA6\n\n";
    let mut reader = decode(Cursor::new(input));
    assert_message(&reader.next().await.unwrap()?, "message", "ok…", None);
    assert!(reader.next().await.is_none());
    Ok(())
}
