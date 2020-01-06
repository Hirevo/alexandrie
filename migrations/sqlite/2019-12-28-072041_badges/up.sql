create table crate_badges (
    id integer primary key,
    crate_id bigint not null,
    badge_type varchar(255) not null,
    params varchar(1024) not null,
    foreign key (crate_id) references crates(id) on update cascade on delete cascade
);
