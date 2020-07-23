use async_std::fs::File;
use async_std::io::{self, Read, SeekFrom, Write};
use async_std::path::PathBuf;
use async_std::sync::Arc;
use async_std::task::{Context, Poll};
use std::pin::Pin;
use std::sync::Mutex;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum Direction {
    Client,
    Server,
}

#[derive(Clone)]
pub struct TestCase {
    direction: Direction,
    source_fixture: Arc<File>,
    expected_fixture: Arc<Mutex<File>>,
    result: Arc<Mutex<File>>,
}

impl TestCase {
    #[allow(dead_code)]
    pub async fn new_server(request_file_path: &str, response_file_path: &str) -> TestCase {
        Self::new(Direction::Server, request_file_path, response_file_path).await
    }

    #[allow(dead_code)]
    pub async fn new_client(request_file_path: &str, response_file_path: &str) -> TestCase {
        Self::new(Direction::Client, request_file_path, response_file_path).await
    }

    async fn new(
        direction: Direction,
        request_file_path: &str,
        response_file_path: &str,
    ) -> TestCase {
        let request_fixture = File::open(fixture_path(&request_file_path))
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Could not open request fixture file: {:?}",
                    &fixture_path(request_file_path)
                )
            });

        let response_fixture = File::open(fixture_path(&response_file_path))
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Could not open response fixture file: {:?}",
                    &fixture_path(response_file_path)
                )
            });

        let temp = tempfile::tempfile().expect("Failed to create tempfile");
        let result = Arc::new(Mutex::new(temp.into()));

        let (source_fixture, expected_fixture) = match direction {
            Direction::Client => (response_fixture, request_fixture),
            Direction::Server => (request_fixture, response_fixture),
        };

        TestCase {
            direction,
            source_fixture: Arc::new(source_fixture),
            expected_fixture: Arc::new(Mutex::new(expected_fixture)),
            result,
        }
    }

    #[allow(dead_code)]
    pub async fn read_result(&self) -> String {
        use async_std::prelude::*;
        let mut result = String::new();
        let mut file = self.result.lock().unwrap();
        file.seek(SeekFrom::Start(0)).await.unwrap();
        file.read_to_string(&mut result).await.unwrap();
        result
    }

    #[allow(dead_code)]
    pub async fn read_expected(&self) -> String {
        use async_std::prelude::*;
        let mut expected = std::string::String::new();
        self.expected_fixture
            .lock()
            .unwrap()
            .read_to_string(&mut expected)
            .await
            .unwrap();
        expected
    }

    #[allow(dead_code)]
    pub(crate) async fn assert(self) {
        let mut actual = self.read_result().await;
        let mut expected = self.read_expected().await;
        assert!(!actual.is_empty(), "Received empty reply");
        assert!(!expected.is_empty(), "Missing expected fixture");

        // munge actual and expected so that we don't rely on dates matching exactly
        munge_date(&mut actual, &mut expected);
        pretty_assertions::assert_eq!(actual, expected);
    }
}

pub(crate) fn fixture_path(relative_path: &str) -> PathBuf {
    let directory: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    directory.join("tests").join(relative_path)
}

pub(crate) fn munge_date(actual: &mut String, expected: &mut String) {
    if let Some(i) = expected.find("{DATE}") {
        match actual.find("date: ") {
            Some(j) => {
                let eol = actual[j + 6..].find("\r\n").expect("missing eol");
                expected.replace_range(i..i + 6, &actual[j + 6..j + 6 + eol]);
            }
            None => {
                expected.replace_range(i..i + 6, "");
            }
        }
    }
}

impl Read for TestCase {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.source_fixture).poll_read(cx, buf)
    }
}

impl Write for TestCase {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.result.lock().unwrap()).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.result.lock().unwrap()).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.result.lock().unwrap()).poll_close(cx)
    }
}
