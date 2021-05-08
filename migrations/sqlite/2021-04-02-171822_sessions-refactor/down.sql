drop table sessions;
create table sessions (
    `id` integer primary key,
    `token` varchar(255) not null unique,
    `author_id` bigint not null,
    `expires` varchar(25) not null,
    foreign key (`author_id`) references `authors`(`id`) on update cascade on delete cascade
);
