# Running Alexandrie with Docker

Alexandrie can be run in a Docker container, which can make it easier to start, stop, and build on a non-Linux system.


## Dependencies

To run Alexandrie in docker, you'll need:

* The `Alexandrie` source pulled from GitHub
* [docker](https://docs.docker.com/install/)
* [docker-compose](https://docs.docker.com/compose/install/)

Make sure that `docker` and `docker-compose` are in your system path.


## User Configuration

A small bit of setup is required before you can start the docker containers. First, copy `example.env` to `.env` (the filename is important, don't prefix the extension), and modify the values inside:

* Create a new, empty directory where the application can create data, and then set `APPDATA` to the absolute path to that folder.
* Set `CRATE_INDEX` to the SSH path of an existing git repository with a valid index `config.json` file.
* Set `GIT_NAME` and `GIT_EMAIL` to valid git values that will be used when Alexandrie commits & pushes those commits to the index.
* Set `GIT_SSH_KEY` to a new or existing passwordless SSH key. The `.pub` key associated with this key should be added to github/gitlab/etc. to grant access to clone and push the crate index.


By default Alexandrie will use SQLite. If you want to use either MySQL or PostgreSQL instead, you'll need to create a file at either `docker/mysqsl/rootpass.txt` or `docker/postgresql/rootpass.txt` which contains the password that will be given to the root user of the database.


## Running


