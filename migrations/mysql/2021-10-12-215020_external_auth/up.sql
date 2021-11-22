alter table `authors` add column `github_id` varchar(255) unique;
alter table `authors` add column `gitlab_id` varchar(255) unique;
alter table `authors` modify column `passwd` varchar(255);
