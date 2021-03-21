#!/bin/bash

# dont allow errors
set -e

# start the ssh agent
eval $(ssh-agent)

# add ssh keys
ssh-add

# this function isolates the hostname of a given URL.
function get_hostname {
    URL="$1"

    # remove the protocol bits of the URL
    URL="${URL#ssh://}"
    URL="${URL#http://}"
    URL="${URL#https://}"

    # Remove the username and/or username:password part of the URL
    URL="${URL#*:*@}"
    URL="${URL#*@}"

    ## Remove the rest of the URL (pathname, query params, hash fragments, etc...)
    URL=${URL%%/*}
    URL=${URL%%:}

    # Show the hostname
    echo "${URL}"
}

# store the hostname of the crate index URL
HOSTNAME="$(get_hostname "${CRATE_INDEX}")"

# add the keys from that host
ssh-keyscan -t rsa "${HOSTNAME}" >> "${HOME}/.ssh/known_hosts"

# configure git with user metadata
git config --global user.name "${GIT_NAME}" && git config --global user.email "${GIT_EMAIL}"

# create the appdata directory if it isn't mounted (this is typically done for end-to-end tests)
mkdir -p appdata

# pull down the crate index, if it doesnt already exist
if [ ! -d appdata/crate-index ]; then
    # git clone $(cat crate-index.txt) crate-index
    git clone ${CRATE_INDEX} appdata/crate-index
fi

# if the crate-storage directory doesn't exist, make it
mkdir -p appdata/crate-storage

# make the appropriate database directory
mkdir -p appdata/${DATABASE}

# for postgres and mysql, on the first run wait for the database to come up,
# then use diesel to set up the database
if [ "${DATABASE}" = "mysql" ] || [ "${DATABASE}" = "postgres" ]; then
    if [ ! -f appdata/${DATABASE}_init_done ]; then
        # wait for the database
        # TODO: fix the URL later with details from alexandrie.toml (could be user & password file),
        # or have alexandrie create the database
        export DATABASE_URL=$(grep "^url" alexandrie.toml | awk '{print substr($3,2,length($3)-2)}')
        if [ "${DATABASE}" = "mysql" ]; then
            export MIGRATION_DIRECTORY=migrations/mysql
        elif [ "${DATABASE}" = "postgres" ]; then
            export MIGRATION_DIRECTORY=migrations/postgres
        fi

        RESULT=1
        # give the database 2 mins to start
        TIMEOUT=120
        ELAPSED=0
        while [ $RESULT -ne 0 ]; do
            set +e
            TMP="$(diesel database setup)"
            RESULT=$?
            set -e
            echo "${TMP}"
            if [ $RESULT -ne 0 ]; then
                sleep 2
                ELAPSED=$(($ELAPSED + 2))
                # if time is up, exit with the last status code
                if [ $ELAPSED -ge $TIMEOUT ]; then
                    exit $RESULT
                fi
            fi
        done

        touch appdata/${DATABASE}_init_done
    fi
fi

# start the server
# export RUST_BACKTRACE=1
# export RUST_LOG=debug
alexandrie
