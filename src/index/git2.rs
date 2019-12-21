use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use semver::{Version, VersionReq};

use crate::error::{AlexError, Error};
use crate::index::{CrateVersion, Indexer};

/// The 'git2' crate index management strategy type.
///
/// It manages the crate index using the [**`libgit2`**][libgit2] library.
///
/// [libgit2]: https://libgit2.org
pub struct Git2Index {
    /// The path of the crate index.
    pub(crate) repo: Mutex<git2::Repository>,
}

impl Git2Index {
    /// Create a Git2Index instance with the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Git2Index, Error> {
        let repo = git2::Repository::open(path)?;
        let repo = Mutex::new(repo);
        Ok(Git2Index { repo })
    }

    fn compute_record_path(&self, name: &str) -> PathBuf {
        let repo = self.repo.lock().unwrap();
        let path = repo.path().parent().unwrap();
        match name.len() {
            1 => path.join("1").join(&name),
            2 => path.join("2").join(&name),
            3 => path.join("3").join(&name[..1]).join(&name),
            _ => path.join(&name[0..2]).join(&name[2..4]).join(&name),
        }
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

impl Indexer for Git2Index {
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

    fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error> {
        let path = self.compute_record_path(name);
        let file = fs::File::open(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(AlexError::CrateNotFound {
                name: String::from(name),
            }),
            _ => Error::from(err),
        })?;
        let found = io::BufReader::new(file)
            .lines()
            .map(|line| Some(json::from_str(line.ok()?.as_str()).ok()?))
            .flat_map(|ret: Option<CrateVersion>| ret.into_iter())
            .filter(|krate| req.matches(&krate.vers))
            .max_by(|k1, k2| k1.vers.cmp(&k2.vers));
        Ok(found.ok_or_else(|| AlexError::CrateNotFound {
            name: String::from(name),
        })?)
    }

    fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        let path = self.compute_record_path(name);
        let reader = io::BufReader::new(fs::File::open(path)?);
        reader
            .lines()
            .map(|line| Ok(json::from_str::<CrateVersion>(line?.as_str())?))
            .collect()
    }

    fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        let records = self.all_records(name)?;
        Ok(records
            .into_iter()
            .max_by(|k1, k2| k1.vers.cmp(&k2.vers))
            .expect("at least one version should exist"))
    }

    fn add_record(&self, record: CrateVersion) -> Result<(), Error> {
        let path = self.compute_record_path(record.name.as_str());

        if let Ok(file) = fs::File::open(&path) {
            let reader = io::BufReader::new(file);
            let records = reader
                .lines()
                .map(|line| Ok(json::from_str::<CrateVersion>(line?.as_str())?))
                .collect::<Result<Vec<CrateVersion>, Error>>()?;
            let latest = records
                .into_iter()
                .max_by(|k1, k2| k1.vers.cmp(&k2.vers))
                .expect("at least one record should exist");
            if record.vers <= latest.vers {
                return Err(Error::from(AlexError::VersionTooLow {
                    krate: record.name,
                    hosted: latest.vers,
                    published: record.vers,
                }));
            }
        } else {
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)?;
        json::to_writer(&mut file, &record)?;
        writeln!(file)?;
        file.flush()?;

        Ok(())
    }

    fn alter_record<F>(&self, name: &str, version: Version, func: F) -> Result<(), Error>
    where
        F: FnOnce(&mut CrateVersion),
    {
        let path = self.compute_record_path(name);
        let file = fs::File::open(path.as_path()).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(AlexError::CrateNotFound {
                name: String::from(name),
            }),
            _ => Error::from(err),
        })?;
        let mut krates: Vec<CrateVersion> = {
            let mut out = Vec::new();
            for line in io::BufReader::new(file).lines() {
                let krate = json::from_str(line?.as_str())?;
                out.push(krate);
            }
            out
        };
        let found = krates
            .iter_mut()
            .find(|krate| krate.vers == version)
            .ok_or_else(|| {
                Error::from(AlexError::CrateNotFound {
                    name: String::from(name),
                })
            })?;

        func(found);

        let lines = krates
            .into_iter()
            .map(|krate| json::to_string(&krate))
            .collect::<Result<Vec<String>, _>>()?;
        fs::write(path.as_path(), lines.join("\n") + "\n")?;

        Ok(())
    }
}
