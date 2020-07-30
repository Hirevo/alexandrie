use crate::error::Error;
use crate::Store;
use lazy_static::lazy_static;
use rusoto_core::Region;
use rusoto_s3::{GetObjectOutput, GetObjectRequest, PutObjectRequest, S3Client, StreamingBody, S3};
use semver::Version;
use std::convert::TryFrom;
use std::fmt;
use std::io::{self, Read};
use tokio::runtime::Runtime;

// Rusoto needs a tokio runtime.
lazy_static! {
    // Seems reasonable to panic here if we can't create a runtime. We'll force
    // this to load in `new` below so we panic on startup if there's a problem.
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

/// The S3-backed storage strategy.
///
/// This mimics the crates.io storage naming. Given a bucket (e.g., "foobar") and a key prefix
#[derive(Clone)]
pub struct S3Storage {
    client: S3Client,
    bucket: String,
    key_prefix: String,
}

impl fmt::Debug for S3Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // Can't get the region back out of S3Client, and it doesn't impl Debug itself :(
        write!(f, "S3Storage {{ bucket: {0}, key_prefix: {1} }}", self.bucket, self.key_prefix)
    }
}

impl S3Storage {
    /// Instantiate a new `S3Storage` handle with the given S3 region, bucket
    /// name, and key prefix.
    pub fn new(region: Region, bucket: String, key_prefix: String) -> Self {
        // Start up the tokio runtime so if something goes wrong we fail fast.
        lazy_static::initialize(&RUNTIME);

        Self {
            client: S3Client::new(region),
            bucket,
            key_prefix,
        }
    }

    /// Generate the S3 bucket key for the given crate name and version.
    pub fn crate_key(&self, name: &str, version: Version) -> String {
        format!("{}/{}/{}-{}.crate", self.key_prefix, name, name, version)
    }

    /// Generate the S3 bucket key for the html-rendered readme page for the
    /// given crate name and version.
    pub fn readme_key(&self, name: &str, version: Version) -> String {
        format!("{}/{}/{}-{}.readme", self.key_prefix, name, name, version)
    }

    fn get_object(&self, key: String) -> Result<GetObjectOutput, Error> {
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            key,
            ..Default::default()
        };
        Ok(RUNTIME.handle().block_on(self.client.get_object(request))?)
    }

    // NOTE: S3 requests can succeed but then give us a body of `None`. I'm not sure
    // what the best way to handle that is - we could convert that into an error,
    // but it's not clear that it actually is an error. Instead, this method and
    // `get_object_reader` below convert "no body" into "no data" and return an
    // empty vec or empty reader.
    fn get_object_data(&self, key: String) -> Result<Vec<u8>, Error> {
        let s3_object = self.get_object(key)?;

        let body = match s3_object.body {
            Some(body) => body,
            None => return Ok(Vec::new()),
        };

        // see if we can preallocate a vec to hold all the data we're about to get
        let mut data = if let Some(content_length) = s3_object
            .content_length
            .and_then(|length| usize::try_from(length).ok())
        {
            Vec::with_capacity(content_length)
        } else {
            Vec::new()
        };

        body.into_blocking_read().read_to_end(&mut data)?;

        Ok(data)
    }

    fn get_object_reader(&self, key: String) -> Result<Box<dyn Read>, Error> {
        let s3_object = self.get_object(key)?;

        // see note on `get_object_data` above on handling `None` here
        let reader: Box<dyn Read> = match s3_object.body {
            Some(body) => Box::new(body.into_blocking_read()),
            None => Box::new(io::empty()),
        };

        Ok(reader)
    }

    fn put_object(&self, key: String, data: Vec<u8>) -> Result<(), Error> {
        let request = PutObjectRequest {
            bucket: self.bucket.clone(),
            key,
            body: Some(StreamingBody::from(data)),
            ..Default::default()
        };

        // Don't think we need any of the data we get back from S3 on a PUT.
        let _output = RUNTIME.handle().block_on(self.client.put_object(request))?;

        Ok(())
    }
}

impl Store for S3Storage {
    fn get_crate(&self, name: &str, version: Version) -> Result<Vec<u8>, Error> {
        self.get_object_data(self.crate_key(name, version))
    }

    fn read_crate(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        self.get_object_reader(self.crate_key(name, version))
    }

    fn store_crate(&self, name: &str, version: Version, data: Vec<u8>) -> Result<(), Error> {
        self.put_object(self.crate_key(name, version), data)
    }

    fn get_readme(&self, name: &str, version: Version) -> Result<String, Error> {
        let data = self.get_object_data(self.readme_key(name, version))?;

        // We're storing READMEs as UTF8 strings, so the lossy conversion here
        // should never lose any data, unless somehow we're getting out some
        // data that was stored externally
        Ok(String::from_utf8_lossy(&data).to_string())
    }

    fn read_readme(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        self.get_object_reader(self.readme_key(name, version))
    }

    fn store_readme(&self, name: &str, version: Version, data: String) -> Result<(), Error> {
        self.put_object(self.readme_key(name, version), data.into_bytes())
    }
}
