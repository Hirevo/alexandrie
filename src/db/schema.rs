table! {
    /// The author table.
    authors (id) {
        /// The ID of the author.
        id -> Unsigned<Bigint>,
        /// The login/username of the author.
        login -> Varchar,
        /// The displayable name of the author.
        name -> Varchar,
        /// The SHA512 hash of the author's password.
        passwd -> Varchar,
    }
}

table! {
    /// The crate metadata table.
    crates (id) {
        /// The ID of the crate.
        id -> Unsigned<Bigint>,
        /// The name of the crate.
        name -> Varchar,
        /// The descripton of the crate.
        description -> Nullable<Varchar>,
        /// The creation date of the crate.
        created_at -> Datetime,
        /// The date of the last update to the crate.
        updated_at -> Datetime,
        /// The download count of the crate.
        downloads -> Unsigned<Bigint>,
        /// The documentation link of the crate.
        documentation -> Nullable<Varchar>,
        /// The repository link of the crate.
        repository -> Nullable<Varchar>,
    }
}

table! {
    /// The crate-to-authors one-to-many relationship table.
    crate_authors (id) {
        /// The ID of the relationship.
        id -> Unsigned<Bigint>,
        /// The crate ID.
        crate_id -> Unsigned<Bigint>,
        /// The author ID.
        author_id -> Unsigned<Bigint>,
    }
}

table! {
    /// The author-to-tokens one-to-many relationship table.
    author_tokens (id) {
        /// The ID of the token.
        id -> Unsigned<Bigint>,
        /// The author ID.
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
