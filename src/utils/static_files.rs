use std::io;
use std::path::{Path, PathBuf};

use futures::compat::Compat01As03 as Compat;
use futures::future::BoxFuture;
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use tide::error::ResultExt;
use tide::response::IntoResponse;
use tide::{Context, Endpoint, Response};

/// The handler for serving static files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticFiles {
    /// The path to the directory to serve from.
    path: PathBuf,
}

impl StaticFiles {
    /// Constructs a `StaticFiles` for the given path.
    pub fn new(path: impl AsRef<Path>) -> io::Result<StaticFiles> {
        Ok(StaticFiles {
            path: path.as_ref().absolutize()?,
        })
    }
}

impl<State: Send + Sync + 'static> Endpoint<State> for StaticFiles {
    type Fut = BoxFuture<'static, Response>;

    fn call(&self, ctx: Context<State>) -> Self::Fut {
        let served = self.path.clone();
        let path = ctx.param::<PathBuf>("path").unwrap();
        let future = async move {
            let path = served.join(path).absolutize().client_err()?;

            if path.starts_with(&served) {
                let metadata = Compat::new(tokio::fs::metadata(path.clone()))
                    .await
                    .server_err()?;
                let bytes = Compat::new(tokio::fs::read(path.clone()))
                    .await
                    .server_err()?;
                let mut builder = http::Response::builder();
                builder.status(http::StatusCode::OK);
                builder.header("content-length", metadata.len());
                if let Some(guess) = mime_guess::from_path(&path).first() {
                    builder.header("content-type", guess.as_ref());
                }
                Ok(builder.body(tide::Body::from(bytes)).unwrap())
            } else {
                Err(tide::error::Error::from(http::StatusCode::NOT_FOUND))
            }
        };

        futures::FutureExt::boxed(async move { future.await.into_response() })
    }
}
