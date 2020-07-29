# Running Alexandrie via Docker

## Introduction
This tutorial walks through setting up a local instance of Alexandrie so you can test publishing a crate to an [alternative registry](https://doc.rust-lang.org/cargo/reference/registries.html#using-an-alternate-registry)

## Dependencies
- Docker

## Steps

### Pull Image
We provide a docker image you can use to run a local instance of Alexandrie. Pull it with
> docker pull rtohaan/alexandrie:latest

This image uses the default Alexandrie configuration and comes with sqlite installed.

Alternatively, you can build it yourself from the root of this repo with
> docker build -t alexandrie -f Dockerfile .

The remaining steps assume you have pulled rtohaan/alexandrie:latest.

### Setting up directories for application data
Alexandrie requires 3 locations to store data. 
- ./crate-storage  
- ./data
- ./crate-index

./crate-storage stores the crate's binaries

./data will contain the sqlite database that Alexandrie needs to keep track of accounts, tokens, etc.

./crate-index stores meta-data for each version of the crate. [Link](https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification) to the alternative-registry spec

These need to be created in the root directory of Alexandrie. *Note: Registries are linked to git repositories*.

To create ./crate-storage
> mkdir crate-storage

To create ./data
> mkdir data

To create ./crate-index
> git clone https://github.com/RangerStation/rangerstation-alexandrie-index.git ./crate-index

### Starting the container
Common flags used with `docker run` 
- -p Binds the port being used in the docker container to a port on your local machine
- -d Runs the docker container in the background
- -v Mounts the docker container to your local filesystem
- -it Makes the docker container interactable through your terminal emulator

Example
```
docker run \
    -it \
    -p 3000:3000 \
    -v $(pwd)/crate-index:/alexandrie/crate-index \
    -v $(pwd)/crate-storage:/alexandrie/crate-storage \
    -v $(pwd)/data:/alexandrie/data \
    rtohaan/alexandrie:latest
```

### Configuring an Alternative Registry
For Alexandrie to locate the alternative registry, a `~/.cargo/config` file needs to be created.

Example:
```toml
[registries.local]
index = "https://github.com/RangerStation/rangerstation-alexandrie-index"
```
Example for configuring a local index repo
```toml
[registries.local]
index = "file://localhost/home/ankit/Documents/Dev/crate-index"
```

### Login to your Registry
Visit `http://localhost:3000/` and on the top right. Register and login to your newly created account.

Visit `http://localhost:3000/account/manage` to create a token for your account.

Copy and Paste your new token, we'll need this to login through `cargo`.
![token](https://i.fluffy.cc/zB4LdrZH8m35LttNmgqdNMqCPgCbGSCp.png)

Next, we need to login to your local registry with
> cargo login --registry local &lt;token&gt;

### Publishing a crate
Lets create a dummy crate to publish to our registry
> cargo new testpackage  && cd testpackage  

Commit the new crate.
> git add . && git commit -m "new crate"

Publish
> cargo publish --registry &lt;registry&gt;


## Conclusion
Congrats you know publish a private crate!
