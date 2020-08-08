alter table crates add column canon_name varchar(255);
update crates set canon_name = replace(name, '-', '_');
alter table crates alter column canon_name set not null;
alter table crates add unique (canon_name);
