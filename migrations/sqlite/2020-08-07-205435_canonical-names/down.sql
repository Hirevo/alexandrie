alter table `crates` rename to `crates_old`;
create table `crates` (
    `id` integer primary key,
    `name` varchar(255) not null unique,
    `description` varchar(4096),
    `created_at` varchar(25) not null,
    `updated_at` varchar(25) not null,
    `downloads` bigint not null default 0,
    `documentation` varchar(1024),
    `repository` varchar(1024)
);
insert into `crates` (`id`, `name`, `description`, `created_at`, `updated_at`, `downloads`, `documentation`, `repository`)
    select `id`, `name`, `description`, `created_at`, `updated_at`, `downloads`, `documentation`, `repository` from `crates_old`;
drop table `crates_old`;
