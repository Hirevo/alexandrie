#!/bin/bash

# dont allow errors
set -e

# start the ssh agent
eval $(ssh-agent)

# add ssh keys
ssh-add

git config --global user.name "${GIT_NAME}" && git config --global user.email "${GIT_EMAIL}"

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
if [ "${DATABASE}" = "mysql" ] || [ "${DATABASE}" = "postgresql" ]; then
    if [ ! -f appdata/${DATABASE}_init_done ]; then
        # wait for the database
        # TODO: fix the URL later with details from alexandrie.toml (could be user & password file),
        # or have alexandrie create the database
        export DATABASE_URL=$(grep "^url" alexandrie.toml | awk '{print substr($3,2,length($3)-2)}')
        if [ "${DATABASE}" = "mysql" ]; then
            export MIGRATION_DIRECTORY=migrations/mysql
        elif [ "${DATABASE}" = "postgresql" ]; then
            export MIGRATION_DIRECTORY=migrations/postgres
        fi

        RESULT=1
        # give the database 2 mins to start
        TIMEOUT=120
        CURTIME=0
        while [ $RESULT -ne 0 ]; do
            TMP="$(diesel database setup)"
            echo ${TMP}
            RESULT=$?
            if [ $RESULT -ne 0 ]; then
                # if 'Connection Refused', wait a bit and try again
                if [ "$( echo $TMP| grep 'Connection Refused')" != "" ]; then
                    delay 2
                    CURTIME=$(($CURTIME + 2))
                    if [ $CURTIME -ge TIMEOUT ]; then
                        exit $RESULT
                    fi
                # for any other error, we don't know what it is so quit trying to connect 
                else
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