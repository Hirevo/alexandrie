alter table "crates" add column "canon_name" varchar(255);
update "crates" set "name" = lower("name"), "canon_name" = replace(lower("name"), '-', '_');
alter table "crates" alter column "canon_name" set not null;
alter table "crates" add unique ("canon_name");
