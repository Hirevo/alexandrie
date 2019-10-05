use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use byteorder::{LittleEndian, ReadBytesExt};
use diesel::connection::Connection;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use flate2::read::GzDecoder;
use futures::stream::StreamExt;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::Archive;
use tide::{Body, Context, Response};

use crate::config::State;
use crate::db::models::{
    CrateAuthor, CrateRegistration, NewCrateAuthor, NewCrateCategory, NewCrateKeyword,
    NewCrateRegistration,
};
use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::index::Indexer;
use crate::krate;
use crate::storage::Store;
use crate::utils;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PublishResponse {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CrateMeta {
    pub name: String,
    pub vers: Version,
    pub deps: Vec<CrateDependency>,
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
struct CrateDependency {
    pub name: String,
    pub version_req: VersionReq,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: Option<krate::DependencyKind>,
    pub registry: Option<String>,
    #[serde(rename = "explicit_name_in_toml")]
    pub explicit_name: Option<String>,
}

async fn read_at_most(body: &mut Body, size: usize) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::with_capacity(size);

    while let Some(chunk) = body.next().await {
        let chunk = chunk?;
        output.extend_from_slice(&chunk);
        if output.len() >= size {
            break;
        }
    }

    output.shrink_to_fit();
    Ok(output)
}

fn link_keywords<Conn>(conn: &Conn, crate_id: u64, keywords: Option<&[String]>) -> Result<(), Error>
where
    Conn: Connection<Backend = Mysql>,
{
    diesel::delete(crate_keywords::table.filter(crate_keywords::crate_id.eq(crate_id)))
        .execute(conn)?;

    if let Some(keywords) = keywords {
        let exprs: Vec<_> = keywords
            .iter()
            .map(|keyword| keywords::name.eq(keyword.as_str()))
            .collect();

        diesel::insert_or_ignore_into(keywords::table)
            .values(&exprs)
            .execute(conn)?;

        let ids = keywords::table
            .select(keywords::id)
            .filter(keywords::name.eq_any(keywords))
            .load::<u64>(conn)?;

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

fn link_categories<Conn>(
    conn: &Conn,
    crate_id: u64,
    categories: Option<&[String]>,
) -> Result<(), Error>
where
    Conn: Connection<Backend = Mysql>,
{
    diesel::delete(crate_categories::table.filter(crate_categories::crate_id.eq(crate_id)))
        .execute(conn)?;

    if let Some(categories) = categories {
        let category_ids = categories::table
            .select(categories::id)
            .filter(categories::tag.eq_any(categories))
            .load::<u64>(conn)?;

        let entries: Vec<_> = category_ids
            .into_iter()
            .map(|category_id| NewCrateCategory {
                crate_id,
                category_id,
            })
            .collect();

        diesel::insert_into(crate_categories::table)
            .values(entries.as_slice())
            .execute(conn)?;
    }

    Ok(())
}

/// Route to publish a new crate (used by `cargo publish`).
pub(crate) async fn put(mut ctx: Context<State>) -> Result<Response, Error> {
    let state = ctx.state();
    let author = state
        .get_author(ctx.headers())
        .await
        .ok_or(AlexError::InvalidToken)?;

    let mut body = ctx.take_body();
    let bytes = read_at_most(&mut body, 10_000_000).await?;
    let mut cursor = io::Cursor::new(&bytes);

    let metadata_size = cursor.read_u32::<LittleEndian>()?;
    let mut metadata_bytes = vec![0u8; metadata_size as usize];
    cursor.read_exact(&mut metadata_bytes)?;
    let metadata: CrateMeta = json::from_slice(&metadata_bytes)?;

    let crate_size = cursor.read_u32::<LittleEndian>()?;
    let mut crate_bytes = vec![0u8; crate_size as usize];
    cursor.read_exact(&mut crate_bytes)?;
    let hash = hex::encode(&Sha256::digest(&crate_bytes));

    let state = ctx.state();
    let repo = &state.repo;

    // state.index.refresh()?;

    let transaction = repo.transaction(|conn| {
        //? Construct a crate description.
        let crate_desc = krate::Crate {
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
                    krate::Dependency {
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
        let result = diesel::insert_into(crates::table)
            .values(new_crate)
            .execute(conn)?;

        //? Fetch the newly inserted (or already existant) crate.
        let krate = crates::table
            .filter(crates::name.eq(crate_desc.name.as_str()))
            .first::<CrateRegistration>(conn)?;

        //? If newly inserted, add the current user as an author.
        //? Else:
        //?  - check if the current user is indeed an author of the crate.
        //?  - check if the version number is higher than the latest stored one.
        //?  - update the crate's metadata.
        let operation = if result == 1 {
            diesel::insert_into(crate_authors::table)
                .values(NewCrateAuthor {
                    crate_id: krate.id,
                    author_id: author.id,
                })
                .execute(conn)?;
            "Adding"
        } else {
            //? Is the user a registered author?
            let not_owned = CrateAuthor::belonging_to(&krate)
                .filter(crate_authors::author_id.eq(&author.id))
                .first::<CrateAuthor>(conn)
                .optional()?
                .is_none();
            if not_owned {
                return Err(Error::from(AlexError::CrateNotOwned(krate.name, author)));
            }

            //? Is the version higher than the latest known one?
            let krate::Crate { vers: latest, .. } =
                state.index.latest_crate(krate.name.as_str())?;
            if crate_desc.vers <= latest {
                return Err(Error::from(AlexError::VersionTooLow {
                    krate: krate.name,
                    hosted: latest,
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
                    crates::updated_at.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)?;
            "Updating"
        };

        //? Update keywords.
        let keywords = metadata.keywords.as_ref().map(|vec| vec.as_slice());
        link_keywords(conn, krate.id, keywords)?;

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
        let path = state.index.index_crate(&crate_desc.name);
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)?;
        json::to_writer(&mut file, &crate_desc)?;
        writeln!(file)?;
        file.flush()?;
        state.index.commit_and_push(&format!(
            "{0} crate `{1}#{2}`",
            operation, &crate_desc.name, &crate_desc.vers
        ))?;

        Ok(tide::response::json(PublishResponse {}))
    });

    transaction.await
}
