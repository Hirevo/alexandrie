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
/// Represents a complete crate entry, as stored in the database.
pub struct CrateRegistration {
    /// The crate's ID.
    pub id: u64,
    /// The crate's name.
    pub name: String,
    /// The crate's description.
    pub description: Option<String>,
    /// The crate's creation date.
    pub created_at: NaiveDateTime,
    /// The crate's last updated date.
    pub updated_at: NaiveDateTime,
    /// The crate's download count.
    pub downloads: u64,
    /// The URL to the crate's documentation.
    pub documentation: Option<String>,
    /// The URL to the crate's repository.
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
/// Represents a partial crate entry from the database,
/// suitable to edit an entry while letting the database maintain the updated date of the row.
pub struct ModifyCrateRegistration<'a> {
    /// The crate's ID.
    pub id: u64,
    /// The crate's name.
    pub name: &'a str,
    /// The crate's description.
    pub description: Option<&'a str>,
    /// The URL to the crate's documentation.
    pub documentation: Option<&'a str>,
    /// The URL to the crate's repository.
    pub repository: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crates"]
/// Represents a partial crate entry from the database,
/// suitable to create an entry while letting the database assign an ID and set the creation date of the row.
pub struct NewCrateRegistration<'a> {
    /// The crate's name.
    pub name: &'a str,
    /// The crate's description.
    pub description: Option<&'a str>,
    /// The URL to the crate's documentation.
    pub documentation: Option<&'a str>,
    /// The URL to the crate's repository.
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
/// Represents a complete author entry, as stored in the database.
pub struct Author {
    /// The author's ID.
    pub id: u64,
    /// The author's email address.
    pub email: String,
    /// The author's displayable name.
    pub name: String,
    /// The author's SHA512-hashed password.
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
    Associations,
    AsChangeset,
)]
#[table_name = "crate_authors"]
#[belongs_to(Author)]
#[belongs_to(CrateRegistration, foreign_key = "crate_id")]
#[primary_key(id)]
/// Represents a crate-to-author relationship in the database.
pub struct CrateAuthor {
    /// The relationship's ID.
    pub id: u64,
    /// The crate's ID.
    pub crate_id: u64,
    /// The author's ID.
    pub author_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crate_authors"]
/// Represents a crate-to-author relationship in the database,
/// suitable to create an entry while letting the database assign a relationship ID.
pub struct NewCrateAuthor {
    /// The crate's ID.
    pub crate_id: u64,
    /// The author's ID.
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
    Associations,
    AsChangeset,
)]
#[table_name = "author_tokens"]
#[belongs_to(Author)]
#[primary_key(id)]
/// Represents a author-to-token relationship in the database.
pub struct AuthorToken {
    /// The token's ID.
    pub id: u64,
    /// The author's ID.
    pub author_id: u64,
    /// The token itself.
    pub token: String,
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
    Associations,
    AsChangeset,
)]
#[table_name = "keywords"]
#[primary_key(id)]
/// Represents a keyword entry in the database.
pub struct Keyword {
    /// The keyword's ID.
    pub id: u64,
    /// The keyword itself.
    pub name: String,
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
    Associations,
    AsChangeset,
)]
#[table_name = "crate_keywords"]
#[belongs_to(Keyword)]
#[belongs_to(CrateRegistration, foreign_key = "crate_id")]
#[primary_key(id)]
/// Represents a crate-to-keyword relationship in the database.
pub struct CrateKeyword {
    /// The relationship's ID.
    pub id: u64,
    /// The crate's ID.
    pub crate_id: u64,
    /// The keyword's ID.
    pub keyword_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crate_keywords"]
/// Represents a crate-to-keyword relationship in the database,
/// suitable to create an entry while letting the database assign a relationship ID.
pub struct NewCrateKeyword {
    /// The crate's ID.
    pub crate_id: u64,
    /// The keyword's ID.
    pub keyword_id: u64,
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
    Associations,
    AsChangeset,
)]
#[table_name = "categories"]
#[primary_key(id)]
/// Represents a category entry in the database.
pub struct Category {
    /// The category's ID.
    pub id: u64,
    /// The category's unique tagname.
    pub tag: String,
    /// The category's name.
    pub name: String,
    /// The category's description.
    pub description: String,
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
    Associations,
    AsChangeset,
)]
#[table_name = "crate_categories"]
#[belongs_to(Category)]
#[belongs_to(CrateRegistration, foreign_key = "crate_id")]
#[primary_key(id)]
/// Represents a crate-to-category relationship in the database.
pub struct CrateCategory {
    /// The relationship's ID.
    pub id: u64,
    /// The crate's ID.
    pub crate_id: u64,
    /// The category's ID.
    pub category_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crate_categories"]
/// Represents a crate-to-category relationship in the database,
/// suitable to create an entry while letting the database assign a relationship ID.
pub struct NewCrateCategory {
    /// The crate's ID.
    pub crate_id: u64,
    /// The category's ID.
    pub category_id: u64,
}
