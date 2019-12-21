use async_std::fs;
use async_std::io;
use async_std::path::{Path, PathBuf};

use futures::future::BoxFuture;
use tide::{Endpoint, IntoResponse, Request, Response, ResultExt};

/// The handler for serving static files.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticFiles {
    /// The path to the directory to serve from.
    path: PathBuf,
}

impl StaticFiles {
    /// Constructs a `StaticFiles` for the given path.
    pub async fn new(path: impl AsRef<Path>) -> io::Result<StaticFiles> {
        let path = path.as_ref();
        let path = if path.is_relative() {
            PathBuf::from(std::env::current_dir()?)
                .join(path)
                .canonicalize()
                .await
        } else {
            path.canonicalize().await
        }?;
        Ok(StaticFiles { path })
    }
}

impl<State: Send + Sync + 'static> Endpoint<State> for StaticFiles {
    type Fut = BoxFuture<'static, Response>;

    fn call(&self, ctx: Request<State>) -> Self::Fut {
        let served = self.path.clone();
        let path = ctx.param::<PathBuf>("path").unwrap();
        let future = async move {
            let path = served.join(path).canonicalize().await.client_err()?;

            if path.starts_with(&served) {
                let metadata = fs::metadata(path.clone()).await.server_err()?;
                let file = fs::File::open(path.clone()).await.server_err()?;
                let mut response = Response::with_reader(200, io::BufReader::new(file))
                    .set_header("content-length", metadata.len().to_string());
                if let Some(guess) = mime_guess::from_path(&path).first() {
                    response = response.set_header("content-type", guess.as_ref());
                }
                Ok(response)
            } else {
                Err(tide::Error::from(http::StatusCode::NOT_FOUND))
            }
        };

        futures::FutureExt::boxed(async move {
            match future.await {
                Ok(response) => response,
                Err(err) => err.into_response(),
            }
        })
    }
}
