/// Upgrade an HTTP connection into an SSE session.
pub fn upgrade(headers: &mut impl AsMut<http_types::Headers>) {
    let headers = headers.as_mut();
    headers.insert("Cache-Control", "no-cache");
    headers.insert("Content-Type", "text/event-stream");
}
