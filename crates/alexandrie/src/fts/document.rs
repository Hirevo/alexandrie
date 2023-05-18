use std::fmt::Formatter;

use tantivy::schema::Schema;
use tantivy::Document;

use crate::error::Error;

/// Represent a crate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TantivyDocument {
    id: i64,
    name: String,
    description: Option<String>,
    readme: Option<String>,
    keywords: Vec<String>,
    categories: Vec<String>,
}

impl std::fmt::Display for TantivyDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, name: {}", self.id, self.name)?;
        if let Some(description) = &self.description {
            write!(f, ", '{}'", description)?;
        }
        // Don't write the README, it might be big, and
        // log will be unreadable. Just say that it has
        // a README.
        if self.readme.is_some() {
            write!(f, ", crate has README",)?;
        }

        if self.keywords.is_empty() {
            write!(f, ", no keyword")?;
        } else {
            let keywords = self.keywords.join(", ");
            write!(f, ", keywords : {keywords}")?;
        }

        if self.categories.is_empty() {
            write!(f, ", no categories")?;
        } else {
            let categories = self.categories.join(", ");
            write!(f, ", categories : {categories}")?;
        }

        Ok(())
    }
}

impl TantivyDocument {
    pub fn new(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            readme: None,
            keywords: Vec::with_capacity(5),
            categories: Vec::with_capacity(5),
        }
    }

    pub fn try_into(self, schema: &Schema) -> Result<Document, Error> {
        // Can't implement From because I need to use a tuple to hold schema and both
        // tuple and Document are in another crate :-(
        let mut document = Document::new();

        let id_field = schema.get_field(super::ID_FIELD_NAME);
        let name_field = schema.get_field(super::NAME_FIELD_NAME);
        let name_full_field = schema.get_field(super::NAME_FIELD_NAME_FULL);
        let name_prefix_field = schema.get_field(super::NAME_FIELD_PREFIX_NAME);
        let description_field = schema.get_field(super::DESCRIPTION_FIELD_NAME);
        let readme_field = schema.get_field(super::README_FIELD_NAME);
        let category_field = schema.get_field(super::CATEGORY_FIELD_NAME);
        let keyword_field = schema.get_field(super::KEYWORD_FIELD_NAME);

        // None of the fields should be `None`.
        // But we check that anyway.
        if id_field.is_none() {
            return Err(Error::MissingField(super::ID_FIELD_NAME));
        }
        if name_field.is_none() {
            return Err(Error::MissingField(super::NAME_FIELD_NAME));
        }
        if name_full_field.is_none() {
            return Err(Error::MissingField(super::NAME_FIELD_NAME_FULL));
        }
        if name_prefix_field.is_none() {
            return Err(Error::MissingField(super::NAME_FIELD_PREFIX_NAME));
        }

        document.add_i64(id_field.unwrap(), self.id);
        document.add_text(name_field.unwrap(), &self.name);
        document.add_text(name_full_field.unwrap(), &self.name);
        document.add_text(name_prefix_field.unwrap(), self.name);

        // For the following fields we will not fail if they are not in schema
        // but TODO add warn
        if let Some(description) = &self.description {
            if let Some(field) = description_field {
                document.add_text(field, description)
            }
        }

        if let Some(readme) = &self.readme {
            if let Some(field) = readme_field {
                document.add_text(field, readme)
            }
        }

        if let Some(field) = keyword_field {
            self.keywords
                .clone()
                .into_iter()
                .for_each(|v| document.add_text(field, v))
        }

        if let Some(field) = category_field {
            self.categories
                .clone()
                .into_iter()
                .for_each(|v| document.add_text(field, v))
        }

        Ok(document)
    }

    /// Set crate's description
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    /// Set crate's README
    pub fn set_readme(&mut self, readme: String) {
        self.readme = Some(readme);
    }

    /// Add new crate's keyword
    pub fn add_keyword(&mut self, keyword: String) {
        self.keywords.push(keyword);
    }

    /// Add new crate's category
    pub fn add_category(&mut self, category: String) {
        self.categories.push(category);
    }
}
