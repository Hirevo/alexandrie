table! {
    /// The author table.
    authors (id) {
        /// The author's ID.
        id -> Unsigned<Bigint>,
        /// The author's email address.
        email -> Varchar,
        /// The author's displayable name.
        name -> Varchar,
        /// The author's SHA512-hashed password.
        passwd -> Varchar,
    }
}

table! {
    /// The crate metadata table.
    crates (id) {
        /// The crate's ID.
        id -> Unsigned<Bigint>,
        /// The crate's name.
        name -> Varchar,
        /// The crate's descripton.
        description -> Nullable<Varchar>,
        /// The crate's creation date.
        created_at -> Datetime,
        /// The crate's last updated date.
        updated_at -> Datetime,
        /// The crate's download count.
        downloads -> Unsigned<Bigint>,
        /// The URL to the crate's documentation.
        documentation -> Nullable<Varchar>,
        /// The URL to the crate's repository.
        repository -> Nullable<Varchar>,
    }
}

table! {
    /// The crate-to-authors (one-to-many) relationship table.
    crate_authors (id) {
        /// The relationship's ID.
        id -> Unsigned<Bigint>,
        /// The crate's ID.
        crate_id -> Unsigned<Bigint>,
        /// The author's ID.
        author_id -> Unsigned<Bigint>,
    }
}

table! {
    /// The author-to-tokens (one-to-many) relationship table.
    author_tokens (id) {
        /// The token's ID.
        id -> Unsigned<Bigint>,
        /// The author's ID.
        author_id -> Unsigned<Bigint>,
        /// The token itself.
        token -> Varchar,
    }
}

table! {
    /// The keywords table (stores all unique keywords).
    keywords (id) {
        /// The keyword's ID.
        id -> Unsigned<Bigint>,
        /// The keyword itself.
        name -> Varchar,
    }
}

table! {
    /// The crate-to-keywords (one-to-many) relationship table.
    crate_keywords (id) {
        /// The relationship's ID.
        id -> Unsigned<Bigint>,
        /// The crate's ID.
        crate_id -> Unsigned<Bigint>,
        /// The keyword's ID.
        keyword_id -> Unsigned<Bigint>,
    }
}

table! {
    /// The categories table (stores all unique categories).
    categories (id) {
        /// The category's ID.
        id -> Unsigned<Bigint>,
        /// The category's unique tagname.
        tag -> Varchar,
        /// The category's name.
        name -> Varchar,
        /// The category's description.
        description -> Varchar,
    }
}

table! {
    /// The crate-to-categories (one-to-many) relationship table.
    crate_categories (id) {
        /// The relationship's ID.
        id -> Unsigned<Bigint>,
        /// The crate's ID.
        crate_id -> Unsigned<Bigint>,
        /// The category's ID.
        category_id -> Unsigned<Bigint>,
    }
}

joinable!(author_tokens -> authors (author_id));
joinable!(crate_authors -> crates (crate_id));
joinable!(crate_authors -> authors (author_id));
joinable!(crate_keywords -> crates (crate_id));
joinable!(crate_keywords -> keywords (keyword_id));
joinable!(crate_categories -> crates (crate_id));
joinable!(crate_categories -> categories (category_id));

allow_tables_to_appear_in_same_query!(
    authors,
    crates,
    keywords,
    categories,
    author_tokens,
    crate_authors,
    crate_keywords,
    crate_categories,
);
