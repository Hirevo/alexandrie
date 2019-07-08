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

joinable!(crate_authors -> authors (author_id));
joinable!(crate_authors -> crates (crate_id));
joinable!(author_tokens -> authors (author_id));

allow_tables_to_appear_in_same_query!(
    authors,
    crates,
    crate_authors,
    author_tokens,
);
