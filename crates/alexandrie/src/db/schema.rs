table! {
    /// The author table.
    authors (id) {
        /// The author's ID.
        id -> Bigint,
        /// The author's email address.
        email -> Varchar,
        /// The author's displayable name.
        name -> Varchar,
        /// The author's SHA512-hashed password.
        passwd -> Nullable<Varchar>,
        /// The author's GitHub user ID.
        github_id -> Nullable<Varchar>,
        /// The author's GitLab user ID.
        gitlab_id -> Nullable<Varchar>,
    }
}

table! {
    /// The crate metadata table.
    crates (id) {
        /// The crate's ID.
        id -> Bigint,
        /// The crate's name.
        name -> Varchar,
        /// The crate's canonical name ('-' are all replaced with '_').
        canon_name -> Varchar,
        /// The crate's descripton.
        description -> Nullable<Varchar>,
        /// The crate's creation date.
        created_at -> Varchar,
        /// The crate's last updated date.
        updated_at -> Varchar,
        /// The crate's download count.
        downloads -> Bigint,
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
        id -> Bigint,
        /// The crate's ID.
        crate_id -> Bigint,
        /// The author's ID.
        author_id -> Bigint,
    }
}

table! {
    /// The author-to-tokens (one-to-many) relationship table.
    author_tokens (id) {
        /// The token's ID.
        id -> Bigint,
        /// The token's name.
        name -> Varchar,
        /// The token itself.
        token -> Varchar,
        /// The author's ID.
        author_id -> Bigint,
    }
}

table! {
    /// The keywords table (stores all unique keywords).
    keywords (id) {
        /// The keyword's ID.
        id -> Bigint,
        /// The keyword itself.
        name -> Varchar,
    }
}

table! {
    /// The crate-to-keywords (one-to-many) relationship table.
    crate_keywords (id) {
        /// The relationship's ID.
        id -> Bigint,
        /// The crate's ID.
        crate_id -> Bigint,
        /// The keyword's ID.
        keyword_id -> Bigint,
    }
}

table! {
    /// The categories table (stores all unique categories).
    categories (id) {
        /// The category's ID.
        id -> Bigint,
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
        id -> Bigint,
        /// The crate's ID.
        crate_id -> Bigint,
        /// The category's ID.
        category_id -> Bigint,
    }
}

table! {
    /// The crate-to-badges (one-to-many) relationship table.
    crate_badges (id) {
        /// The relationship's ID.
        id -> Bigint,
        /// The crate's ID.
        crate_id -> Bigint,
        /// The badge's type.
        badge_type -> Varchar,
        /// The badge's parameters (as JSON).
        params -> Varchar,
    }
}

table! {
    /// The user sessions table.
    sessions (id) {
        /// The session's ID.
        id -> Varchar,
        /// The session's related author ID.
        author_id -> Nullable<Bigint>,
        /// The session's expiry date.
        expiry -> Varchar,
        /// The session's associated data.
        data -> Text,
    }
}

table! {
    /// The user password salts table.
    salts (id) {
        /// The salt's ID.
        id -> Bigint,
        /// The salt itself.
        salt -> Varchar,
        /// The salt's related author ID.
        author_id -> Bigint,
    }
}

joinable!(author_tokens -> authors (author_id));
joinable!(crate_authors -> crates (crate_id));
joinable!(crate_authors -> authors (author_id));
joinable!(crate_keywords -> crates (crate_id));
joinable!(crate_keywords -> keywords (keyword_id));
joinable!(crate_categories -> crates (crate_id));
joinable!(crate_categories -> categories (category_id));
joinable!(crate_badges -> crates (crate_id));
joinable!(sessions -> authors (author_id));
joinable!(salts -> authors (author_id));

allow_tables_to_appear_in_same_query!(
    authors,
    crates,
    keywords,
    categories,
    author_tokens,
    crate_authors,
    crate_keywords,
    crate_categories,
    crate_badges,
    sessions,
    salts,
);
