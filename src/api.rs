use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use byteorder::{LittleEndian, ReadBytesExt};
use diesel::prelude::*;
use diesel::result::Error as SQLError;
use rocket::{Data, State};
use rocket_contrib::json::Json;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::db::models::{CrateRegistration, ModifyCrateRegistration, NewCrateRegistration};
use crate::db::schema::*;
use crate::{
    AlexError, AppState, Auth, Crate, DbConn, Dependency, DependencyKind, Error, Indexer, Store,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIPublishResponse {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APISearchResponse {
    pub crates: Vec<APISearchResult>,
    pub meta: APISearchMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APISearchResult {
    pub name: String,
    pub max_version: Version,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APISearchMeta {
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APICrateMeta {
    pub name: String,
    pub vers: Version,
    pub deps: Vec<APICrateDependency>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APICrateDependency {
    pub optional: bool,
    pub default_features: bool,
    pub name: String,
    pub features: Vec<String>,
    pub version_req: VersionReq,
    pub target: Option<String>,
    pub kind: Option<DependencyKind>,
}

#[put("/crates/new", data = "<data>")]
pub fn api_publish(
    state: State<Arc<Mutex<AppState>>>,
    _auth: Auth,
    conn: DbConn,
    data: Data,
) -> Result<Json<APIPublishResponse>, Error> {
    let mut stream = data.open();
    let metadata_size = stream.read_u32::<LittleEndian>()?;
    let mut metadata_bytes = vec![0u8; metadata_size as usize];
    stream.read_exact(&mut metadata_bytes)?;
    let metadata: APICrateMeta = json::from_slice(&metadata_bytes)?;
    let crate_size = stream.read_u32::<LittleEndian>()?;
    let mut crate_bytes = vec![0u8; crate_size as usize];
    stream.read_exact(&mut crate_bytes)?;
    let hash = hex::encode(&Sha256::digest(&crate_bytes));

    let state = state.lock().unwrap();
    state.index().refresh()?;
    let krate = crates::table
        .filter(crates::name.eq(metadata.name.as_str()))
        .first::<CrateRegistration>(&conn.0)
        .optional()?;
    if let Some(krate) = krate {
        let max_version = state.index().max_version(krate.name.as_str())?;
        if metadata.vers <= max_version {
            Err(Error::from(AlexError::VersionTooLow {
                krate: krate.name,
                hosted: max_version,
                published: metadata.vers,
            }))
        } else {
            state.storage().store_crate(
                &metadata.name,
                metadata.vers.clone(),
                crate_bytes.as_slice(),
            )?;

            let path = state.index().index_crate(&metadata.name);
            let crate_desc = Crate {
                name: metadata.name,
                vers: metadata.vers,
                deps: metadata
                    .deps
                    .into_iter()
                    .map(|dep| Dependency {
                        name: dep.name,
                        req: dep.version_req,
                        features: dep.features,
                        optional: dep.optional,
                        default_features: dep.default_features,
                        target: dep.target,
                        kind: dep.kind,
                    })
                    .collect(),
                cksum: hash,
                features: metadata.features,
                yanked: Some(false),
            };
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;
            let mut file = fs::OpenOptions::new().write(true).append(true).open(path)?;
            json::to_writer(&mut file, &crate_desc)?;
            write!(file, "\n")?;
            file.flush()?;
            state.index().commit_and_push(&format!(
                "Updating crate `{}#{}`",
                &crate_desc.name, &crate_desc.vers
            ))?;

            let new_crate = ModifyCrateRegistration {
                id: krate.id,
                name: crate_desc.name.as_str(),
                description: metadata.description.as_ref().map(|s| s.as_str()),
                documentation: metadata.documentation.as_ref().map(|s| s.as_str()),
                repository: metadata.repository.as_ref().map(|s| s.as_str()),
            };
            diesel::update(crates::table)
                .set(new_crate)
                .execute(&conn.0)?;

            Ok(Json(APIPublishResponse {}))
        }
    } else {
        state.storage().store_crate(
            &metadata.name,
            metadata.vers.clone(),
            crate_bytes.as_slice(),
        )?;

        let path = state.index().index_crate(metadata.name.as_str());
        let crate_desc = Crate {
            name: metadata.name,
            vers: metadata.vers,
            deps: metadata
                .deps
                .into_iter()
                .map(|dep| Dependency {
                    name: dep.name,
                    req: dep.version_req,
                    features: dep.features,
                    optional: dep.optional,
                    default_features: dep.default_features,
                    target: dep.target,
                    kind: dep.kind,
                })
                .collect(),
            cksum: hash,
            features: metadata.features,
            yanked: Some(false),
        };
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        json::to_writer(&mut file, &crate_desc)?;
        write!(file, "\n")?;
        file.flush()?;
        state.index().commit_and_push(&format!(
            "Adding crate `{}#{}`",
            &crate_desc.name, &crate_desc.vers
        ))?;

        let new_crate = NewCrateRegistration {
            name: crate_desc.name.as_str(),
            description: metadata.description.as_ref().map(|s| s.as_str()),
            documentation: metadata.documentation.as_ref().map(|s| s.as_str()),
            repository: metadata.repository.as_ref().map(|s| s.as_str()),
        };
        diesel::insert_into(crates::table)
            .values(new_crate)
            .execute(&conn.0)?;
        Ok(Json(APIPublishResponse {}))
    }
}

#[get("/crates?<q>&<per_page>&<page>")]
pub fn api_search(
    state: State<Arc<Mutex<AppState>>>,
    conn: DbConn,
    q: String,
    per_page: Option<u32>,
    page: Option<u32>,
) -> Result<Json<APISearchResponse>, Error> {
    let state = state.lock().unwrap();
    state.index().refresh()?;
    let name_pattern = format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%"));
    let req = crates::table
        .select((crates::name, crates::description))
        .filter(crates::name.like(name_pattern.as_str()))
        .into_boxed();
    let req = match (per_page, page) {
        (Some(per_page), Some(page)) => req.limit(per_page as i64).offset((page * per_page) as i64),
        (Some(per_page), None) => req.limit(per_page as i64),
        _ => req,
    };
    let results = req.load::<(String, Option<String>)>(&conn.0)?;
    let total = crates::table
        .select(diesel::dsl::count(crates::name))
        .filter(crates::name.like(name_pattern.as_str()))
        .first::<i64>(&conn.0)?;

    let crates = results
        .into_iter()
        .map(|(name, description)| {
            let max_version = state.index().max_version(name.as_str())?;
            Ok(APISearchResult {
                name,
                max_version,
                description,
            })
        })
        .collect::<Result<Vec<APISearchResult>, Error>>()?;

    Ok(Json(APISearchResponse {
        crates,
        meta: APISearchMeta { total: total as u64 },
    }))
}

#[get("/crates/<name>/<version>/download")]
pub fn api_download(
    state: State<Arc<Mutex<AppState>>>,
    _auth: Auth,
    conn: DbConn,
    name: String,
    version: String,
) -> Result<Vec<u8>, Error> {
    let version = Version::parse(&version)?;
    let state = state.lock().unwrap();
    state.index().refresh()?;
    let krate = state.storage().get_crate(&name, version)?;
    let downloads = crates::table
        .select(crates::downloads)
        .filter(crates::name.eq(name.as_str()))
        .first::<u64>(&conn.0)?;
    diesel::update(crates::table)
        .set(crates::downloads.eq(downloads + 1))
        .execute(&conn.0)?;
    Ok(krate)
}
