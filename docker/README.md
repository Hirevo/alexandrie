# Docker Image Tutorial
Short tutorial on getting your Docker Image up and running. We will be assuming that you're using a SQLite database. Link to [Docker Installation](https://docs.docker.com/get-docker/)

## Building the Image
Manual build
> docker build -t &lt;Image Name&gt; -f $PWD/Dockerfile .

Pulling the Image from Docker Hub
> docker pull rtohaan/alexandrie:latest

## Changing the Port
In Alexandrie, the port is set to 3000 by default. Edit alexandrie.toml to change it to your prefered settings.
```toml
[general]
addr = "0.0.0.0"
port = 3000
```

## Storage
Alexandrie needs 3 places to store data: /crate-index, /crate-storage, and /data where the SQLite database will be stored. The Docker Image will auto-generate these folders but we need to mount them to your local filesystem using the [-v flag](https://docs.docker.com/engine/reference/run/#volume-shared-filesystems) 

## Docker Run
The full command should look like this:
> docker run -it -v $(pwd)/crate-index:/alexandrie/crate-index -v $(pwd)/crate-storage:/alexandrie/crate-storage -v $(pwd)/data:/alexandrie/data &lt;Image Name&gt;