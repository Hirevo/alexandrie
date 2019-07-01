use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use semver::Version;

use crate::{AlexError, Crate, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    path: PathBuf,
}

impl Index {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Result<Index, Error> {
        let path = dir.into();

        Ok(Index { path })
    }

    pub fn refresh(&self) -> Result<(), Error> {
        Command::new("git")
            .arg("pull")
            .arg("--ff-only")
            .current_dir(self.path.canonicalize()?)
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn url(&self) -> Result<String, Error> {
        let output = Command::new("git")
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .stdout(Stdio::piped())
            .current_dir(self.path.canonicalize()?)
            .output()?;

        Ok(String::from_utf8_lossy(output.stdout.as_slice()).into())
    }

    pub fn index_crate(&self, name: &str) -> PathBuf {
        match name.len() {
            1 => self.path.join("1").join(&name),
            2 => self.path.join("2").join(&name),
            3 => self.path.join("3").join(&name[..1]).join(&name),
            _ => self.path.join(&name[0..2]).join(&name[2..4]).join(&name),
        }
    }

    pub fn find_crate(&self, name: &str, version: Version) -> Result<Crate, Error> {
        let path = self.index_crate(name);
        let file = fs::File::open(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(AlexError::CrateNotFound(String::from(name))),
            _ => Error::from(err),
        })?;
        let found = io::BufReader::new(file)
            .lines()
            .map(|line| Some(json::from_str(line.ok()?.as_str()).ok()?))
            .flat_map(|ret: Option<Crate>| ret.into_iter())
            .find(|krate| krate.vers == version);
        Ok(found.ok_or_else(|| AlexError::CrateNotFound(String::from(name)))?)
    }

    pub fn search_crates(&self, q: &str, limit: Option<u32>) -> Result<Vec<Crate>, Error> {
        let mut names = Vec::with_capacity(10);
        for dir in fs::read_dir(&self.path)? {
            if let Ok(dir) = dir {
                let name = dir.file_name();

                let iter = fs::read_dir(self.path.join("1"))?
                    .flat_map(|dir| dir.ok().into_iter())
                    .chain(fs::read_dir(self.path.join("2"))?.flat_map(|dir| dir.ok().into_iter()))
                    .chain(fs::read_dir(self.path.join("3"))?.flat_map(|dir| dir.ok().into_iter()))
                    .chain({
                        fs::read_dir(&self.path)?
                            .flat_map(|el| el.ok().into_iter())
                            .filter(|dir| dir.file_type().map_or_else(|_| false, |ty| ty.is_dir()))
                            .flat_map(|dir| fs::read_dir(dir.path()).ok().into_iter().flatten())
                            .flat_map(|el| el.ok().into_iter())
                            .filter(|dir| dir.file_type().map_or_else(|_| false, |ty| ty.is_dir()))
                            .flat_map(|dir| fs::read_dir(dir.path()).ok().into_iter().flatten())
                            .flat_map(|el| el.ok().into_iter())
                            .filter(|dir| dir.file_type().map_or_else(|_| false, |ty| ty.is_file()))
                            .filter(|dir| {
                                dir.file_name()
                                    .to_str()
                                    .map_or(false, |name| name.starts_with(q))
                            })
                    });

                if let Some(limit) = limit {
                    return Ok(iter
                        .take(limit as usize)
                        .map(|dir| {
                            let file = fs::File::open(dir.path()).map_err(|err| match err.kind() {
                                io::ErrorKind::NotFound => {
                                    Error::from(AlexError::CrateNotFound(String::from(q)))
                                }
                                _ => Error::from(err),
                            })?;
                            Ok(json::from_reader(file)?)
                        })
                        .filter(|ret: &Result<Crate, Error>| ret.is_ok())
                        .map(|ret| ret.unwrap())
                        .collect());
                } else {
                    return Ok(iter
                        .map(|dir| {
                            let file = fs::File::open(dir.path()).map_err(|err| match err.kind() {
                                io::ErrorKind::NotFound => {
                                    Error::from(AlexError::CrateNotFound(String::from(q)))
                                }
                                _ => Error::from(err),
                            })?;
                            Ok(json::from_reader(file)?)
                        })
                        // .filter(|ret: &Result<Crate, Error>| ret.is_ok())
                        .map(|ret: Result<Crate, Error>| ret.unwrap())
                        .collect());
                }

                if name == "1" {
                    if q.len() <= 1 {
                        for dir in fs::read_dir(self.path.join("1"))? {
                            if let Ok(dir) = dir {
                                if dir
                                    .file_name()
                                    .to_str()
                                    .map_or(false, |name| name.starts_with(q))
                                {
                                    names.push(dir.path());
                                }
                            }
                        }
                    }
                } else if name == "2" {
                    if q.len() <= 2 {
                        for dir in fs::read_dir(self.path.join("2"))? {
                            if let Ok(dir) = dir {
                                if dir
                                    .file_name()
                                    .to_str()
                                    .map_or(false, |name| name.starts_with(q))
                                {
                                    names.push(dir.path());
                                }
                            }
                        }
                    }
                } else if name == "3" {
                    if q.len() <= 3 {
                        for dir in fs::read_dir(self.path.join("3"))? {
                            if let Ok(dir) = dir {
                                if let Some(name) = dir.file_name().to_str() {
                                    if name[0..1].starts_with(&q[0..1]) {
                                        for dir in
                                            fs::read_dir(self.path.join("3").join(&name[0..1]))?
                                        {
                                            if let Ok(dir) = dir {
                                                if dir
                                                    .file_name()
                                                    .to_str()
                                                    .map_or(false, |name| name.starts_with(q))
                                                {
                                                    names.push(dir.path());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    if let Some(name) = name.to_str() {
                        if q.len() < 3 || name[0..2].starts_with(&q[0..2]) {
                            for dir in fs::read_dir(self.path.join(&name[0..2]))? {
                                if let Ok(dir) = dir {
                                    if let Some(name2) = dir.file_name().to_str() {
                                        if q.len() < 5 || name2[0..2].starts_with(&q[2..4]) {
                                            for dir in fs::read_dir(
                                                self.path.join(&name[0..2]).join(&name2[0..2]),
                                            )? {
                                                if let Ok(dir) = dir {
                                                    if dir
                                                        .file_name()
                                                        .to_str()
                                                        .map_or(false, |name| name.starts_with(q))
                                                    {
                                                        names.push(dir.path());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        dbg!(&names);

        if let Some(limit) = limit {
            Ok(names
                .into_iter()
                .take(limit as usize)
                .map(|name| {
                    let file = fs::File::open(name).map_err(|err| match err.kind() {
                        io::ErrorKind::NotFound => {
                            Error::from(AlexError::CrateNotFound(String::from(q)))
                        }
                        _ => Error::from(err),
                    })?;
                    Ok(json::from_reader(file)?)
                })
                .filter(|ret: &Result<Crate, Error>| ret.is_ok())
                .map(|ret| ret.unwrap())
                .collect())
        } else {
            Ok(names
                .into_iter()
                .map(|name| {
                    let file = fs::File::open(name).map_err(|err| match err.kind() {
                        io::ErrorKind::NotFound => {
                            Error::from(AlexError::CrateNotFound(String::from(q)))
                        }
                        _ => Error::from(err),
                    })?;
                    Ok(json::from_reader(file)?)
                })
                // .filter(|ret: &Result<Crate, Error>| ret.is_ok())
                .map(|ret: Result<Crate, Error>| ret.unwrap())
                .collect())
        }
    }
}
