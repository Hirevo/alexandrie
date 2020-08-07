#
# Dockerfile for the Alexandrie crate registry application
#

### First stage: build the application
FROM rust:1.50-slim-buster as builder

ARG DATABASE

RUN apt update
RUN apt install -y clang libssl-dev pkg-config
# install proper dependencies for each database
# for postgresql and mysql, install diesel as well to set up the database
# for sqlite make a dummy file for Docker to copy
RUN \
    if [ "${DATABASE}" = "sqlite" ]; then \
        apt install -y sqlite3 libsqlite3-dev; \
        mkdir -p /usr/local/cargo/bin/; \
        touch /usr/local/cargo/bin/diesel; \
    fi && \
    if [ "${DATABASE}" = "postgres" ]; then \
        apt install -y  libpq-dev; \
        cargo install diesel_cli --no-default-features --features "postgres"; \
    fi && \
    if [ "${DATABASE}" = "mysql" ]; then \
        apt install -y default-libmysqlclient-dev; \
        cargo install diesel_cli --no-default-features --features "mysql"; \
    fi

WORKDIR /alexandrie

# copy source data
COPY crates crates
COPY syntect syntect
COPY helpers helpers
COPY migrations migrations
COPY wasm-pbkdf2 wasm-pbkdf2
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# build the app
RUN cd crates/alexandrie && cargo build --release --no-default-features --features "${DATABASE} frontend git2"

### Second stage: copy built application
FROM debian:buster-slim as runner

ARG DATABASE

# install run dependencies, then clean up apt cache
RUN apt update && \
    apt install -y openssh-client git && \
    if [ "${DATABASE}" = "sqlite" ]; then apt install -y sqlite3; fi && \
    if [ "${DATABASE}" = "postgres" ]; then apt install -y  postgresql; fi && \
    if [ "${DATABASE}" = "mysql" ]; then apt install -y default-mysql-server default-mysql-client; fi && \
    apt-get clean && rm -rf /var/lib/apt/lists/

# copy run files
COPY --from=builder /alexandrie/target/release/alexandrie /usr/bin/alexandrie
# copy docker_cli
COPY --from=builder /usr/local/cargo/bin/diesel /usr/bin/diesel
# add the startup file
COPY docker/startup.sh /home/alex/startup.sh
# copy runtime assets
COPY assets /home/alex/assets
COPY syntect /home/alex/syntect
COPY templates /home/alex/templates
COPY migrations /home/alex/migrations
# copy diesel config
# COPY diesel.toml /home/alex/diesel.toml

ARG USER_ID=1000
ARG GROUP_ID=1000

# combine run instructions to reduce docker layers & overall image size
RUN \
    # make a non-root user
    groupadd -g ${GROUP_ID} alex && \
    useradd -u ${USER_ID} -g ${GROUP_ID} alex && \
    # make the user directory & give them access to everything in it
    # mkdir -p /home/alex && \
    mkdir -p /home/alex/.ssh && \
    chown -R ${USER_ID}:${GROUP_ID} /home/alex && \
    # give alex ownership of diesel
    chown ${USER_ID}:${GROUP_ID} /usr/bin/diesel && \
    # give alex ownership of the startup script & make it executable
    chmod +x /home/alex/startup.sh

# switch to the non-root user to run the main process
USER alex
WORKDIR /home/alex

CMD ./startup.sh
