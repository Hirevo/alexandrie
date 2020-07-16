#
# Dockerfile for the Alexandrie crate registry application
#

FROM rust:1.40-slim-buster as builder

RUN apt update
RUN apt install -y clang libssl-dev pkg-config
# install proper dependencies for each database
# for postgresql and mysql, install diesel as well to set up the database
# for sqlite make a dummy file for Docker to copy
RUN apt install -y sqlite3 libsqlite3-dev
# Cruft we might want
#mkdir -p /usr/local/cargo/bin/; \
#        touch /usr/local/cargo/bin/diesel;

WORKDIR /alexandrie

# Copy everything from docker context into current working dir of docker image being built
COPY ./ ./

RUN pwd
RUN ls

# build the app
RUN cargo build --release
