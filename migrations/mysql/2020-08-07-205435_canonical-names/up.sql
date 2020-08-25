alter table crates add column canon_name varchar(255);
update crates set name = lower(name), canon_name = replace(lower(name), '-', '_');
alter table crates modify canon_name varchar(255) not null;
alter table crates add unique (canon_name);
