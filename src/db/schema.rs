table! {
    authors (id) {
        id -> Unsigned<Bigint>,
        login -> Varchar,
        name -> Varchar,
        passwd -> Varchar,
    }
}

table! {
    crates (id) {
        id -> Unsigned<Bigint>,
        name -> Varchar,
        description -> Nullable<Varchar>,
        created_at -> Datetime,
        updated_at -> Datetime,
        downloads -> Unsigned<Bigint>,
        documentation -> Nullable<Varchar>,
        repository -> Nullable<Varchar>,
    }
}

table! {
    crate_authors (id) {
        id -> Unsigned<Bigint>,
        crate_id -> Unsigned<Bigint>,
        author_id -> Unsigned<Bigint>,
    }
}

table! {
    author_tokens (id) {
        id -> Unsigned<Bigint>,
        author_id -> Unsigned<Bigint>,
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
