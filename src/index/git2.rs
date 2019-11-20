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

impl Indexer for Git2Index {
    fn url(&self) -> Result<String, Error> {
        let repo = self.repo.lock().unwrap();
        let remote = repo.find_remote("origin")?;
        Ok(remote.url().map_or_else(String::default, String::from))
    }

    fn refresh(&self) -> Result<(), Error> {
        return unimplemented!();

        let repo = self.repo.lock().unwrap();
        let mut remote = repo.find_remote("origin")?;
        let mut opts = git2::FetchOptions::new();
        let mut callbacks = git2::RemoteCallbacks::new();
        // TODO: configure RemoteCallbacks properly.
        opts.remote_callbacks(callbacks);
        remote.fetch(&["master"], Some(&mut opts), None)?;

        let ours = {
            let head = repo.head()?;
            head.peel_to_commit()?
        };
        let theirs = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            fetch_head.peel_to_commit()?
        };

        let mut opts = git2::MergeOptions::new();
        opts.fail_on_conflict(true);

        let mut index = repo.merge_commits(&ours, &theirs, Some(&opts))?;

        index.write()?;

        Ok(())
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
        remote.push(&[], None)?;

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
