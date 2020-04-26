use crate::Error;

/// the [`Repository`] trait is implemented by types which can manage a git repository.
pub trait Repository {
    fn url(&self) -> Result<String, Error>;

    fn refresh(&self) -> Result<(), Error>;

    fn commit_and_push(&self, msg: &str) -> Result<(), Error>;
}
