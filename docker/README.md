# Docker Image Tutorial
Short tutorial on getting your Docker Image up and running. Link to [Docker Installation](https://docs.docker.com/get-docker/)

## Building the Image
---------------------
This will take a few minutes.
> docker build -t &lt;Image Name&gt; -f $PWD/Dockerfile .

## Docker Run
-------------
In Alexandrie, the port is set to 3000 by default. Edit alexandrie.toml to change it to your prefered settings.

> docker run -p &lt;PORT&gt;:3000 &lt;Image Name&gt;