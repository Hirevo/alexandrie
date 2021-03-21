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


By default Alexandrie will use SQLite. If you want to use either MySQL or PostgreSQL instead, you'll need to create a file at either `docker/mysql/rootpass.txt` or `docker/postgres/rootpass.txt` which contains the password that will be given to the root user of the database.


### Additional Configuration

If necessary, `alexandrie.toml` and even `diesel.toml` can still be modified, and the docker images can be configured to use those modified files instead. You should read the [Internals](#internals) section first, and will likely need to already have docker knowledge. Some other config files and scripts will need to be modified if you change Alexandrie's port or appdata mount location, for example.


## Usage

### Running

To run, you can use the `run_docker.sh` script for easy setup or teardown. See `run_docker.sh --help` for a list of options.

By default, Alexandrie will use SQLite and run in the background. 

**Bringup**
```bash
./run_docker.sh up
```

If you want to use MySQL and run in the foreground, here's an example:

```bash
./run_docker.sh up --mysql -f
```

If you run in the foreground and kill the services with `Ctrl+C`, the docker containers will stop, but you still may need to run [teardown](#stopping-and-cleanup) for the containers to be deleted fully.


### Stopping and Cleanup

Stopping Alexandrie is as easy as starting it.

**Teardown**
```bash
./run_docker.sh down
```

To stop, for example, a MySQL setup for Alexandrie, run the following:

```bash
./run_docker.sh down --mysql
```

If you're not planning on running Alexandrie again or want to do a full clean of the environment, you can disassociate your local appdata storage from the docker volume database by doing `docker volume prune`, which will delete all unused volumes.


### Next Steps

As Alexandrie will (by default) serve on port `3000` (and directly setting that to port `80` will likely not work for a non-root user and won't allow https), the recommended way to serve the application would be to install & configure `nginx` or another ingress controller.


## Internals

The `run_docker` script is using `docker-compose` under the hood, which is itself just `docker` instrumentation. `docker-compose` is configured for this project through a few files:

* `.env`: contains variables that can be changed by the user for easy configuration
* `docker-compose.yaml`: basic setup for docker images, volumes, etc.; should rarely be touched by the user
* `docker/<database>/<database>-compose.yaml`: supplemental and database-dependent; should rarely be touched by the user

Some additional files are responsible for actually creating the Docker image for Alexandrie, as well as handling starting the application, database, etc.:

* `Dockerfile`: definition for creating the Docker image for Alexandrie itself
* `docker/startup.sh`: ran inside the Docker image to do first-run database initialization with diesel, ssh & git configuration, and start Alexandrie
* `docker/<database>/alexandrie.toml`: application configuration, which has database-dependent features


Modifying `alexandrie.toml` may require additional modification of some of these files, for example if the port is modified.


It's worth mentioning that the Docker image will copy the Alexandrie source contained in the local directory, a.k.a. the source isn't pulled down from the git repo when building the image. If you modifiy the source code, those modifications can be used to make the image for testing, etc.


### Example Run

To start Alexandrie in Docker without the `run_docker` script, you could do:

```bash
export DATABASE=mysql
docker-compose -f docker-compose.yaml -f docker/mysql/mysql-compose.yaml up
```

This will use all of the following files, in addition to the Alexandrie source code:
* `docker-compose.yaml`
* `.env`
* `diesel.toml`
* `Dockerfile`
* `docker/startup.sh`
* `docker/mysql/mysql-compose.yaml`
* `docker/mysql/alexandrie.toml`
* `docker/mysql/rootpass.txt`


### Database

Databases will put their data in an appropriately named directory inside the user-defined `APPDATA` directory (see [User Configuration](#user-configuration)). If the directory already exists, it must either **(a)** have valid database data inside, or **(b)** be empty. Directly modifying files inside the database directory isn't recommended, and can cause the server to fail to start entirely. Don't add extra files or create a file inside an empty database directory; the database will almost certainly complain.
