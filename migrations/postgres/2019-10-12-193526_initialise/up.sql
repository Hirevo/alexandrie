create table authors (
    id bigserial primary key,
    email varchar(255) not null unique,
    name varchar(255) not null,
    passwd varchar(255) not null
);
create table crates (
    id bigserial primary key,
    name varchar(255) not null unique,
    description varchar(4096),
    created_at varchar(25) not null,
    updated_at varchar(25) not null,
    downloads bigint not null default 0,
    documentation varchar(1024),
    repository varchar(1024)
);
create table keywords (
    id bigserial primary key,
    name varchar(255) not null unique
);
create table categories (
    id bigserial primary key,
    tag varchar(255) not null unique,
    name varchar(255) not null,
    description varchar(512) not null
);
create table sessions (
    id bigserial primary key,
    token varchar(255) not null unique,
    author_id bigint not null,
    expires varchar(25) not null,
    foreign key (author_id) references authors(id) on update cascade on delete cascade
);
create table salts (
    id bigserial primary key,
    salt varchar(255) not null,
    author_id bigint not null unique,
    foreign key (author_id) references authors(id) on update cascade on delete cascade
);
create table author_tokens (
    id bigserial primary key,
    name varchar(255) not null,
    token varchar(255) not null unique,
    author_id bigint not null,
    foreign key (author_id) references authors(id) on update cascade on delete cascade
);
create table crate_authors (
    id bigserial primary key,
    crate_id bigint not null,
    author_id bigint not null,
    foreign key (crate_id) references crates(id) on update cascade on delete cascade,
    foreign key (author_id) references authors(id) on update cascade on delete cascade
);
create table crate_categories (
    id bigserial primary key,
    crate_id bigint not null,
    category_id bigint not null,
    foreign key (crate_id) references crates(id) on update cascade on delete cascade,
    foreign key (category_id) references categories(id) on update cascade on delete cascade
);
create table crate_keywords (
    id bigserial primary key,
    crate_id bigint not null,
    keyword_id bigint not null,
    foreign key (crate_id) references crates(id) on update cascade on delete cascade,
    foreign key (keyword_id) references keywords(id) on update cascade on delete cascade
);
