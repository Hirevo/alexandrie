#
# Dockerfile for the Alexandrie crate registry application
#

FROM rust:1.40-slim-buster as builder

RUN apt update
RUN apt install -y clang libssl-dev pkg-config

# Install sqlite
RUN apt install -y sqlite3 libsqlite3-dev

# Install git and configure a default "dev@localhost" user
RUN apt install git -y
RUN git config --global user.email "dev@localhost" && git config --global user.name "dev"

WORKDIR /alexandrie

# Copy everything from docker context into current working dir of docker image being built
COPY ./ ./

# Creates relevant application data dirs if not exists
RUN mkdir -p data
RUN mkdir -p crate-index
RUN mkdir -p crate-storage

RUN pwd
RUN ls

# Build the app
RUN cargo build --release

# Set the default command to run on container start
CMD cargo run --release
