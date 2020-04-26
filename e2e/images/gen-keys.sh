#!/bin/bash

# don't allow errors
set -e

# delete old keys
rm -rf keys

# create directory for the keys
mkdir -p keys

# generate new keys into that directory
ssh-keygen -t rsa -f ./keys/id_rsa -q -N ""
