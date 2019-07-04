use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::db::schema::*;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
)]
#[table_name = "crates"]
#[primary_key(id)]
pub struct CrateRegistration {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub downloads: u64,
    pub documentation: Option<String>,
    pub repository: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
)]
#[table_name = "crates"]
#[primary_key(id)]
pub struct ModifyCrateRegistration<'a> {
    pub id: u64,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub documentation: Option<&'a str>,
    pub repository: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crates"]
pub struct NewCrateRegistration<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub documentation: Option<&'a str>,
    pub repository: Option<&'a str>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
)]
#[table_name = "authors"]
#[primary_key(id)]
pub struct Author {
    pub id: u64,
    pub login: String,
    pub name: String,
    pub passwd: String,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
)]
#[table_name = "crate_authors"]
#[belongs_to(CrateRegistration)]
#[belongs_to(Author)]
#[primary_key(id)]
pub struct CrateAuthor {
    pub id: u64,
    pub crate_id: u64,
    pub author_id: u64,
}


#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
)]
#[table_name = "author_tokens"]
#[belongs_to(Author)]
#[primary_key(id)]
pub struct AuthorToken {
    pub id: u64,
    pub author_id: u64,
    pub token: String,
}
