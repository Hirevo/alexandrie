alter table `authors` rename to `authors_old`;
create table `authors` (
    `id` integer primary key,
    `email` varchar(255) not null unique,
    `name` varchar(255) not null,
    `passwd` varchar(255) unique,
);
insert into `authors` (`id`, `email`, `name`, `passwd`)
    select `id`, `email`, `name`, `passwd` from `authors_old`;
drop table `authors_old`;
