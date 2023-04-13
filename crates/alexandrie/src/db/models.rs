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
pub struct Crate {
    /// The crate's ID.
    pub id: i64,
    /// The crate's name.
    pub name: String,
    /// The crate's canonical name ('-' are all replaced with '_').
    pub canon_name: String,
    /// The crate's description.
    pub description: Option<String>,
    /// The crate's creation date.
    pub created_at: String,
    /// The crate's last updated date.
    pub updated_at: String,
    /// The crate's download count.
    pub downloads: i64,
    /// The URL to the crate's documentation.
    pub documentation: Option<String>,
    /// The URL to the crate's repository.
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crates"]
/// Represents a partial crate entry from the database,
/// suitable to create an entry while letting the database assign it an ID.
pub struct NewCrate<'a> {
    /// The crate's name.
    pub name: &'a str,
    /// The crate's canonical name ('-' are all replaced with '_').
    pub canon_name: &'a str,
    /// The crate's description.
    pub description: Option<&'a str>,
    /// The crate's creation date.
    pub created_at: &'a str,
    /// The crate's last updated date.
    pub updated_at: &'a str,
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
    pub id: i64,
    /// The author's email address.
    pub email: String,
    /// The author's displayable name.
    pub name: String,
    /// The author's SHA512-hashed password.
    pub passwd: Option<String>,
    /// The author's GitHub user ID.
    pub github_id: Option<String>,
    /// The author's GitLab user ID.
    pub gitlab_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "authors"]
/// Represents an author in the database,
/// suitable to create an entry while letting the database assign an author ID.
pub struct NewAuthor<'a> {
    /// The author's email address.
    pub email: &'a str,
    /// The author's displayable name.
    pub name: &'a str,
    /// The author's SHA512-hashed password.
    pub passwd: Option<&'a str>,
    /// The author's GitHub user ID.
    pub github_id: Option<&'a str>,
    /// The author's GitLab user ID.
    pub gitlab_id: Option<&'a str>,
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
#[belongs_to(Crate, foreign_key = "crate_id")]
#[primary_key(id)]
/// Represents a crate-to-author relationship in the database.
pub struct CrateAuthor {
    /// The relationship's ID.
    pub id: i64,
    /// The crate's ID.
    pub crate_id: i64,
    /// The author's ID.
    pub author_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crate_authors"]
/// Represents a crate-to-author relationship in the database,
/// suitable to create an entry while letting the database assign a relationship ID.
pub struct NewCrateAuthor {
    /// The crate's ID.
    pub crate_id: i64,
    /// The author's ID.
    pub author_id: i64,
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
    pub id: i64,
    /// The token's name.
    pub name: String,
    /// The token itself.
    pub token: String,
    /// The token's related author ID.
    pub author_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "author_tokens"]
/// Represents a author-to-token relationship in the database,
/// suitable to create an entry while letting the database assign a relationship ID.
pub struct NewAuthorToken<'a> {
    /// The token's name.
    pub name: &'a str,
    /// The token itself.
    pub token: &'a str,
    /// The token's related author ID.
    pub author_id: i64,
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
    pub id: i64,
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
#[belongs_to(Crate, foreign_key = "crate_id")]
#[primary_key(id)]
/// Represents a crate-to-keyword relationship in the database.
pub struct CrateKeyword {
    /// The relationship's ID.
    pub id: i64,
    /// The crate's ID.
    pub crate_id: i64,
    /// The keyword's ID.
    pub keyword_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crate_keywords"]
/// Represents a crate-to-keyword relationship in the database,
/// suitable to create an entry while letting the database assign a relationship ID.
pub struct NewCrateKeyword {
    /// The crate's ID.
    pub crate_id: i64,
    /// The keyword's ID.
    pub keyword_id: i64,
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
    pub id: i64,
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
#[belongs_to(Crate, foreign_key = "crate_id")]
#[primary_key(id)]
/// Represents a crate-to-category relationship in the database.
pub struct CrateCategory {
    /// The relationship's ID.
    pub id: i64,
    /// The crate's ID.
    pub crate_id: i64,
    /// The category's ID.
    pub category_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crate_categories"]
/// Represents a crate-to-category relationship in the database,
/// suitable to create an entry while letting the database assign a relationship ID.
pub struct NewCrateCategory {
    /// The crate's ID.
    pub crate_id: i64,
    /// The category's ID.
    pub category_id: i64,
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
#[table_name = "crate_badges"]
#[belongs_to(Crate, foreign_key = "crate_id")]
#[primary_key(id)]
/// Represents a crate's badge in the database.
pub struct Badge {
    /// The badge's ID.
    pub id: i64,
    /// The crate's ID.
    pub crate_id: i64,
    /// The badge's type.
    pub badge_type: String,
    /// The badge's parameters.
    pub params: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "crate_badges"]
/// Represents a crate's badge in the database.
/// suitable to create an entry while letting the database assign it an ID.
pub struct NewBadge {
    /// The crate's ID.
    pub crate_id: i64,
    /// The badge's type.
    pub badge_type: String,
    /// The badge's parameters.
    pub params: String,
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
#[table_name = "sessions"]
#[belongs_to(Author)]
#[primary_key(id)]
/// Represents a session in the database.
pub struct Session {
    /// The session's ID.
    pub id: String,
    /// The session's related author ID.
    pub author_id: Option<i64>,
    /// The session's expiry date.
    pub expiry: String,
    /// The session's associated data.
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "sessions"]
/// Represents a session in the database,
/// suitable to create an entry while letting the database assign it an ID.
pub struct NewSession {
    /// The session's related author ID.
    pub author_id: i64,
    /// The session's expiry date.
    pub expiry: String,
    /// The session's associated data.
    pub data: String,
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
#[table_name = "salts"]
#[belongs_to(Author)]
#[primary_key(id)]
/// Represents a salt in the database.
pub struct Salt {
    /// The salt's ID.
    pub id: i64,
    /// The salt itself.
    pub salt: String,
    /// The salt's related author ID.
    pub author_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "salts"]
/// Represents a salt in the database,
/// suitable to create an entry while letting the database assign it an ID.
pub struct NewSalt<'a> {
    /// The salt itself.
    pub salt: &'a str,
    /// The salt's related author ID.
    pub author_id: i64,
}
