#!/bin/bash


function print_help() {
    echo "start_docker.sh"
    echo "Starts Alexandrie in a Docker container, with a configurable database & other"
    echo "options. Make sure you've followed the README to set up Docker config before "
    echo "running this"
    echo ""
    echo "(* == selected by default)"
    echo "Flags:"
    echo "    --build      : Rebuild the docker images. Usually used if .env variables,"
    echo "                   Alexandrie source, or Cargo.toml are updated. Not required"
    echo "                   for Alexandrie.toml updates"
    echo "    --cleanbuild : Does a docker-compose --build --nocache. Used if something"
    echo "                   didn't build right the first time, perhaps due to network"
    echo "                   issues"
    echo "    -d           : Start docker in daemon (background) mode"
    echo "    -f           : Start docker in foreground mode (Ctrl+C to exit)"
    echo "Database Options:"
    echo "    --sqlite *   : Start Alexandrie with the sqlite configuration files"
    echo "    --mysql      : Start Alexandrie with the mysql configuration files"
    echo "    --postgresql : Start Alexandrie with the postgresql configuration files"
}

DATABASE=sqlite
DO_BUILD=" "
DO_CLEAN=" "
DAEMON="-d"

while [ "$#" -gt 0 ]; do
    case "$1" in
        "--sqlite") DATABASE=sqlite; shift;;
        "--mysql") DATABASE=mysql; shift;;
        "--postgresql") DATABASE=postgresql; shift;;
        "--build") DO_BUILD="--build"; shift;;
        "--cleanbuild") DO_CLEAN="true"; shift;;
        "--help"|"-h"|"help") print_help; exit 0;;
        *) print_help; exit 1;;
    esac
done

# determine what docker-compose config files to use
if [ "$DATABASE" = "sqlite" ]; then
    FILES="-f docker-compose.yaml"
elif [ "$DATABASE" = "mysql" ]; then
    FILES="-f docker-compose.yaml -f docker/mysql/mysql-compose.yaml"
elif [ "$DATABASE" = "postgresql" ]; then
    FILES="-f docker-compose.yaml -f docker/postgresql/postgresql-compose.yaml"
else
    echo "Invalid database specified. How did you get here?"
    exit 1
fi

# run docker-compose, building if specified
if [ "$DO_CLEAN" = "true" ]; then
    DATABASE=$DATABASE docker-compose ${FILES} build --no-cache
    DATABASE=$DATABASE docker-compose ${FILES} up
else
    DATABASE=$DATABASE docker-compose ${FILES} up ${DO_BUILD}
fi

