alter table "authors" add column "github_id" varchar(255) unique;
alter table "authors" add column "gitlab_id" varchar(255) unique;
alter table "authors" alter column "passwd" type varchar(255);
