pub mod cli;
#[cfg(feature = "git2")]
pub mod git2;
use crate::error::Error;

pub trait Repository {
    fn url(&self) -> Result<String, Error>;

    fn refresh(&self) -> Result<(), Error>;

    fn commit_and_push(&self, msg: &str) -> Result<(), Error>;
}
