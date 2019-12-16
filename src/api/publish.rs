use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

use async_std::io::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};
use diesel::dsl as sql;
use diesel::prelude::*;
use flate2::read::GzDecoder;
use ring::digest as hasher;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tar::Archive;
use tide::{Request, Response};

use crate::db::models::{
    CrateRegistration, NewCrateAuthor, NewCrateCategory, NewCrateKeyword, NewCrateRegistration,
};
use crate::db::schema::*;
use crate::db::Connection;
use crate::db::DATETIME_FORMAT;
use crate::error::{AlexError, Error};
use crate::index::{CrateDependency, CrateDependencyKind, CrateVersion, Indexer};
use crate::storage::Store;
use crate::utils;
use crate::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PublishResponse {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CrateMeta {
    pub name: String,
    pub vers: Version,
    pub deps: Vec<CrateMetaDependency>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub links: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CrateMetaDependency {
    pub name: String,
    pub version_req: VersionReq,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: Option<CrateDependencyKind>,
    pub registry: Option<String>,
    #[serde(rename = "explicit_name_in_toml")]
    pub explicit_name: Option<String>,
}

fn link_keywords(
    conn: &Connection,
    crate_id: i64,
    keywords: Option<&[String]>,
) -> Result<(), Error> {
    diesel::delete(crate_keywords::table.filter(crate_keywords::crate_id.eq(crate_id)))
        .execute(conn)?;

    if let Some(keywords) = keywords {
        let exprs: Vec<_> = keywords
            .iter()
            .map(|keyword| keywords::name.eq(keyword.as_str()))
            .collect();

        #[cfg(any(feature = "mysql", feature = "sqlite"))]
        diesel::insert_or_ignore_into(keywords::table)
            .values(&exprs)
            .execute(conn)?;

        #[cfg(feature = "postgres")]
        diesel::insert_into(keywords::table)
            .values(&exprs)
            .on_conflict_do_nothing()
            .execute(conn)?;

        let ids = keywords::table
            .select(keywords::id)
            .filter(keywords::name.eq_any(keywords))
            .load::<i64>(conn)?;

        let entries: Vec<_> = ids
            .into_iter()
            .map(|keyword_id| NewCrateKeyword {
                crate_id,
                keyword_id,
            })
            .collect();

        diesel::insert_into(crate_keywords::table)
            .values(entries.as_slice())
            .execute(conn)?;
    }

    Ok(())
}

fn link_categories(
    conn: &Connection,
    crate_id: i64,
    categories: Option<&[String]>,
) -> Result<(), Error> {
    diesel::delete(crate_categories::table.filter(crate_categories::crate_id.eq(crate_id)))
        .execute(conn)?;

    if let Some(categories) = categories {
        let category_ids = categories::table
            .select(categories::id)
            .filter(categories::tag.eq_any(categories))
            .load::<i64>(conn)?;

        let entries: Vec<_> = category_ids
            .into_iter()
            .map(|category_id| NewCrateCategory {
                crate_id,
                category_id,
            })
            .collect();

        diesel::insert_into(crate_categories::table)
            .values(&entries)
            .execute(conn)?;
    }

    Ok(())
}

/// Route to publish a new crate (used by `cargo publish`).
pub(crate) async fn put(mut req: Request<State>) -> Result<Response, Error> {
    let headers = req.headers().clone();
    let state = req.state().clone();
    let repo = &state.repo;
    let author = repo
        .run(move |conn| utils::checks::get_author(conn, &headers))
        .await
        .ok_or(AlexError::InvalidToken)?;

    let mut bytes = Vec::new();
    (&mut req).take(10_000_000).read_to_end(&mut bytes).await?;
    let mut cursor = std::io::Cursor::new(&bytes);

    let metadata_size = cursor.read_u32::<LittleEndian>()?;
    let mut metadata_bytes = vec![0u8; metadata_size as usize];
    cursor.read_exact(&mut metadata_bytes)?;
    let metadata: CrateMeta = json::from_slice(&metadata_bytes)?;

    let crate_size = cursor.read_u32::<LittleEndian>()?;
    let mut crate_bytes = vec![0u8; crate_size as usize];
    cursor.read_exact(&mut crate_bytes)?;
    let hash = hex::encode(hasher::digest(&hasher::SHA256, &crate_bytes).as_ref());

    let repo = &state.repo;

    // state.index.refresh()?;

    let transaction = repo.transaction(move |conn| {
        let state = req.state();

        //? Construct a crate description.
        let crate_desc = CrateVersion {
            name: metadata.name,
            vers: metadata.vers,
            deps: metadata
                .deps
                .into_iter()
                .map(|dep| {
                    let (name, package) = if let Some(renamed) = dep.explicit_name {
                        (renamed, Some(dep.name))
                    } else {
                        (dep.name, None)
                    };
                    CrateDependency {
                        name,
                        req: dep.version_req,
                        features: dep.features,
                        optional: dep.optional,
                        default_features: dep.default_features,
                        target: dep.target,
                        kind: dep.kind,
                        registry: dep.registry,
                        package,
                    }
                })
                .collect(),
            cksum: hash,
            features: metadata.features,
            yanked: Some(false),
            links: metadata.links,
        };

        //? Attempt to insert the new crate.
        let new_crate = NewCrateRegistration {
            name: crate_desc.name.as_str(),
            description: metadata.description.as_ref().map(|s| s.as_str()),
            documentation: metadata.documentation.as_ref().map(|s| s.as_str()),
            repository: metadata.repository.as_ref().map(|s| s.as_str()),
        };

        //? Does the crate already exists?
        let exists = utils::checks::crate_exists(conn, new_crate.name)?;

        //? Are we adding a new crate or updating a new one?
        let operation = if exists {
            "Updating"
        } else {
            //? Insert the new crate (as it doesn't already exists).
            diesel::insert_into(crates::table)
                .values(new_crate)
                .execute(conn)?;
            "Adding"
        };

        //? Fetch the newly inserted (or already existant) crate.
        let krate: CrateRegistration = crates::table
            .filter(crates::name.eq(crate_desc.name.as_str()))
            .first(conn)?;

        //? If newly inserted, add the current user as an author.
        //? Else:
        //?  - check if the current user is an author of the crate: if not, emit error.
        //?  - check if the version number is higher than the latest stored one: if not, emit error.
        //?  - update the crate's metadata.
        if exists {
            //? Is the user an author of this crate?
            let owned: bool = sql::select(sql::exists(
                crate_authors::table
                    .filter(crate_authors::crate_id.eq(&krate.id))
                    .filter(crate_authors::author_id.eq(&author.id)),
            ))
            .get_result(conn)?;
            if !owned {
                return Err(Error::from(AlexError::CrateNotOwned {
                    author,
                    name: krate.name,
                }));
            }

            //? Is the version higher than the latest known one?
            let latest = state.index.latest_record(krate.name.as_str())?;
            if crate_desc.vers <= latest.vers {
                return Err(Error::from(AlexError::VersionTooLow {
                    krate: krate.name,
                    hosted: latest.vers,
                    published: crate_desc.vers,
                }));
            }

            //? Update the crate's metadata.
            let description = metadata.description.as_ref().map(|s| s.as_str());
            let documentation = metadata.documentation.as_ref().map(|s| s.as_str());
            let repository = metadata.repository.as_ref().map(|s| s.as_str());
            diesel::update(crates::table.filter(crates::id.eq(krate.id)))
                .set((
                    crates::description.eq(description),
                    crates::documentation.eq(documentation),
                    crates::repository.eq(repository),
                    crates::updated_at.eq(chrono::Utc::now()
                        .naive_utc()
                        .format(DATETIME_FORMAT)
                        .to_string()
                        .as_str()),
                ))
                .execute(conn)?;
        } else {
            //? Insert the current user as an initial author of the crate.
            diesel::insert_into(crate_authors::table)
                .values(NewCrateAuthor {
                    crate_id: krate.id,
                    author_id: author.id,
                })
                .execute(conn)?;
        };

        //? Update keywords.
        let keywords = metadata.keywords.as_ref().map(|vec| vec.as_slice());
        link_keywords(conn, krate.id, keywords)?;

        //? Update categories.
        let categories = metadata.categories.as_ref().map(|vec| vec.as_slice());
        link_categories(conn, krate.id, categories)?;

        //? Store the crate's tarball.
        state.storage.store_crate(
            &crate_desc.name,
            crate_desc.vers.clone(),
            crate_bytes.as_slice(),
        )?;

        //? Render the crate's readme.
        let mut archive = Archive::new(GzDecoder::new(crate_bytes.as_slice()));
        let base_path = PathBuf::from(format!("{0}-{1}", crate_desc.name, crate_desc.vers));
        let readme_path = base_path.join("README.md");
        let mut entries = archive.entries()?;
        let found = entries.find(|entry| match entry {
            Ok(entry) => entry
                .path()
                .map(|path| path == readme_path)
                .unwrap_or(false),
            Err(_) => false,
        });

        //? Start render if it has a README.
        if let Some(found) = found {
            let mut contents = String::new();
            found?.read_to_string(&mut contents)?;

            let rendered = utils::rendering::render_readme(&state.syntect, contents.as_str());

            state.storage.store_readme(
                &crate_desc.name,
                crate_desc.vers.clone(),
                rendered.as_bytes(),
            )?;
        }

        //? Update the crate index.
        let commit_msg = format!(
            "{0} crate `{1}#{2}`",
            operation,
            crate_desc.name.as_str(),
            &crate_desc.vers
        );
        state.index.add_record(crate_desc)?;
        state.index.commit_and_push(commit_msg.as_str())?;

        Ok(utils::response::json(&PublishResponse {}))
    });

    transaction.await
}
