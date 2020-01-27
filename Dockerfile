#
# Dockerfile for the Alexandrie crate registry application
#
# The output docker image will assume the default Cargo.toml options
# (i.e., sqlite3 database)
#

### First stage: build the application
FROM rust as builder

# for now, assume that we'll be using sqlite
RUN apt update
RUN apt install -y sqlite3 clang

WORKDIR /alexandrie

# copy source data
COPY assets assets
COPY migrations migrations
COPY src src
# COPY syntect-dumps syntect-dumps
COPY syntect-syntaxes syntect-syntaxes
COPY syntect-themes syntect-themes
COPY templates templates
COPY wasm-pbkdf2 wasm-pbkdf2
COPY Cargo.toml Cargo.toml

# build the app
RUN cargo build --release


### Second stage: copy built application
FROM debian:buster-slim as runner

# install run dependencies
RUN apt update
RUN apt install -y sqlite3 openssh-client git


# copy run files
COPY --from=builder /alexandrie/target/release/alexandrie /usr/bin/alexandrie

# make a non-root user
RUN groupadd -g 1000 alex && useradd -u 1000 -g 1000 alex

# make the user directory & give them access
RUN mkdir /home/alex && chown -R alex:alex /home/alex
RUN mkdir /home/alex/.ssh && chown alex:alex /home/alex/.ssh

# add a nameserver for dns handling
RUN echo "nameserver 8.8.8.8" >> /etc/resolv.conf

# add the startup file
COPY startup.sh /home/alex/startup.sh
RUN chown alex:alex /home/alex/startup.sh && chmod +x /home/alex/startup.sh

# switch to the non-root user to run the main process
USER alex
WORKDIR /home/alex

# copy runtime assets
COPY assets assets
COPY syntect-dumps syntect-dumps
COPY templates templates

# to make this as much of a one-stop-shop as possible, copy in the diesel config
COPY diesel.toml diesel.toml

# make sure github is in the list of known hosts
# we'll do this at build time, rather than every run time
RUN ssh-keyscan -t rsa github.com >> ~/.ssh/known_hosts

# start the ssh-agent, install the git ssh key, and 
# RUN eval $(ssh-agent) && \
#     ssh-add git_ssh_key && \

# clone the index repo into `crate-index`
# RUN git clone $(cat crate-index.txt) crate-index

CMD ./startup.sh
