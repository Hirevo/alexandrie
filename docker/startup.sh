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
if [ ! -d appdata/crate-storage ]; then
    mkdir appdata/crate-storage
fi

# start the server
# export RUST_BACKTRACE=1
# export RUST_LOG=debug
alexandrie