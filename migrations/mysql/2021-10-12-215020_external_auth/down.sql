alter table `authors` drop column `github_id`;
alter table `authors` drop column `gitlab_id`;
alter table `authors` modify column `passwd` varchar(255) unique;
