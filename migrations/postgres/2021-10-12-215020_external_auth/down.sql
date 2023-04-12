alter table "authors" drop column "github_id";
alter table "authors" drop column "gitlab_id";
alter table "authors" alter column "passwd" varchar(255) unique;
alter table "authors" alter column "passwd" set not null;
