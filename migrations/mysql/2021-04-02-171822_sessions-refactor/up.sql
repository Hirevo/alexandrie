drop table sessions;
create table sessions (
    `id` varchar(255) not null unique primary key,
    `author_id` bigint,
    `expiry` varchar(25) not null,
    `data` text not null,
    foreign key (`author_id`) references `authors`(`id`) on update cascade on delete cascade
);
