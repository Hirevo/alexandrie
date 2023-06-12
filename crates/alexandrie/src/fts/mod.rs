//! Full-text search module
mod document;
mod index;

pub(crate) use document::TantivyDocument;
pub(crate) use index::Tantivy;

/// Default number of result per page
/// Perhaps should make this configurable in toml.
pub const DEFAULT_RESULT_PER_PAGE: usize = 15;

/// Database ID.
const ID_FIELD_NAME: &str = "id";
/// Tokenized version of crate's name.
const NAME_FIELD_NAME: &str = "name";
/// Another index of crate's name, this one
/// isn't tokenized. So it's an exact match
/// but case-insensitive. It's here to improve
/// results relevance.
const NAME_FIELD_NAME_FULL: &str = "name.full";
/// A third index for crate's name. This one is for
/// suggestion in the search bar. It's tokenized and
/// contains word's prefixes to do "search as you type".
const NAME_FIELD_PREFIX_NAME: &str = "name.prefix";
const DESCRIPTION_FIELD_NAME: &str = "description";
const CATEGORY_FIELD_NAME: &str = "category";
const KEYWORD_FIELD_NAME: &str = "keyword";
