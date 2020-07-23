use async_sse::{decode, encode, Event};
use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::task;
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
async fn encode_message() -> http_types::Result<()> {
    let (sender, encoder) = encode();
    task::spawn(async move {
        sender.send("cat", "chashu", None).await;
    });

    let mut reader = decode(BufReader::new(encoder));
    let event = reader.next().await.unwrap()?;
    assert_message(&event, "cat", "chashu", None);
    Ok(())
}

#[async_std::test]
async fn encode_message_with_id() -> http_types::Result<()> {
    let (sender, encoder) = encode();
    task::spawn(async move {
        sender.send("cat", "chashu", Some("0")).await;
    });

    let mut reader = decode(BufReader::new(encoder));
    let event = reader.next().await.unwrap()?;
    assert_message(&event, "cat", "chashu", Some("0"));
    Ok(())
}

#[async_std::test]
async fn encode_retry() -> http_types::Result<()> {
    let (sender, encoder) = encode();
    task::spawn(async move {
        let dur = Duration::from_secs(12);
        sender.send_retry(dur, None).await;
    });

    let mut reader = decode(BufReader::new(encoder));
    let event = reader.next().await.unwrap()?;
    assert_retry(&event, 12);
    Ok(())
}
