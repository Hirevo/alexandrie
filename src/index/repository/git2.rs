use super::Repository;
use crate::error::Error;
use git2;
use std::path::Path;
use std::sync::Mutex;

/// The 'git2' crate index management strategy type.
///
/// It manages the crate index using the [**`libgit2`**][libgit2] library.
///
/// [libgit2]: https://libgit2.org
pub struct Repo {
    /// The path of the crate index.
    repo: Mutex<git2::Repository>,
}

impl Repo {
    /// Create a Repository instance with the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let repo = Mutex::new(git2::Repository::open(path)?);
        Ok(Self { repo })
    }
}

/// Helper to run git operations that require authentication.
///
/// This is inspired by [the way Cargo handles this][cargo-impl].
///
/// [cargo-impl]: https://github.com/rust-lang/cargo/blob/94bf4781d0bbd266abe966c6fe1512bb1725d368/src/cargo/sources/git/utils.rs#L437
fn with_credentials<F, T>(repo: &git2::Repository, mut f: F) -> Result<T, git2::Error>
where
    F: FnMut(&mut git2::Credentials) -> Result<T, git2::Error>,
{
    let config = repo.config()?;

    let mut tried_sshkey = false;
    let mut tried_cred_helper = false;
    let mut tried_default = false;

    f(&mut |url, username, allowed| {
        if allowed.contains(git2::CredentialType::USERNAME) {
            return Err(git2::Error::from_str("no username specified in remote URL"));
        }

        if allowed.contains(git2::CredentialType::SSH_KEY) && !tried_sshkey {
            tried_sshkey = true;
            let username = username.unwrap();
            return git2::Cred::ssh_key_from_agent(username);
        }

        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) && !tried_cred_helper {
            tried_cred_helper = true;
            return git2::Cred::credential_helper(&config, url, username);
        }

        if allowed.contains(git2::CredentialType::DEFAULT) && !tried_default {
            tried_default = true;
            return git2::Cred::default();
        }

        Err(git2::Error::from_str("no authentication methods succeeded"))
    })
}

impl Repository for Repo {
    fn url(&self) -> Result<String, Error> {
        let repo = self.repo.lock().unwrap();
        let remote = repo.find_remote("origin")?;
        Ok(remote.url().map_or_else(String::default, String::from))
    }

    fn refresh(&self) -> Result<(), Error> {
        let repo = self.repo.lock().unwrap();
        let mut remote = repo.find_remote("origin")?;
        let branch = repo
            .branches(Some(git2::BranchType::Local))?
            .flatten()
            .map(|(branch, _)| branch)
            .find(|branch| branch.is_head())
            .ok_or_else(|| git2::Error::from_str("detached HEAD not supported"))?;
        let branch_name = branch.name()?.expect("branch name is invalid UTF-8");

        with_credentials(&repo, |cred_callback| {
            let mut opts = git2::FetchOptions::new();
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(cred_callback);
            opts.remote_callbacks(callbacks);
            remote.fetch(&[branch_name], Some(&mut opts), None)?;
            Ok(())
        })?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;
        if analysis.is_up_to_date() {
            Ok(())
        } else if analysis.is_fast_forward() {
            let refname = format!("refs/heads/{}", branch_name);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            Ok(())
        } else {
            Err(Error::from(git2::Error::from_str("fast-forward only!")))
        }
    }

    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        let repo = self.repo.lock().unwrap();
        let oid = {
            let mut index = repo.index()?;
            index.add_all(&["."], git2::IndexAddOption::DEFAULT, None)?;
            index.write_tree()?
        };
        let signature = repo.signature()?;
        let tree = repo.find_tree(oid)?;
        let parent = {
            let head = repo.head()?;
            head.peel_to_commit()?
        };

        repo.commit(Some("HEAD"), &signature, &signature, msg, &tree, &[&parent])?;

        let mut remote = repo.find_remote("origin")?;
        remote.push::<&'static str>(&[], None)?;

        Ok(())
    }
}
